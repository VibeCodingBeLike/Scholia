#![allow(dead_code)]
#![windows_subsystem = "windows"]
use eframe::egui;
use std::path::{Path, PathBuf};

mod export;
mod fonts;
mod keybinds;
mod mla_rules;
mod model;
mod theme;
mod ui;

use export::{export_to_docx, export_to_html, export_to_text};
use keybinds::{Action, KeybindConfig};
use model::{to_mla_title_case, MlaDocument};
use theme::ThemeConfig;
use ui::{
    render_calendar_popup, render_citation_modal, render_compliance_modal, render_editor_page,
    render_settings_modal, render_status_bar, render_toolbar, render_works_cited_modal,
    CalendarModalState, CitationModalState, ComplianceModalState, EditorAction, SettingsModalState,
    ToolbarEvent, WorksCitedModalState,
};

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_transparent(true)
            .with_decorations(true)
            .with_inner_size([1120.0, 860.0])
            .with_min_inner_size([720.0, 500.0])
            .with_title("Scholia — MLA 9th Edition Document Editor"),
        ..Default::default()
    };

    eframe::run_native(
        "Scholia",
        native_options,
        Box::new(|cc| {
            fonts::configure_fonts(&cc.egui_ctx);
            Ok(Box::new(MlaApp::new()))
        }),
    )
}

struct MlaApp {
    doc: MlaDocument,
    theme: ThemeConfig,
    keybinds: KeybindConfig,

    // Modal states
    works_cited_state: WorksCitedModalState,
    citation_state: CitationModalState,
    compliance_state: ComplianceModalState,
    settings_state: SettingsModalState,
    calendar_state: CalendarModalState,

    focus_mode: bool,
    vibrancy_dirty: bool,
    notification: Option<(String, std::time::Instant)>,
}

impl MlaApp {
    pub fn new() -> Self {
        // Load saved theme & keybinds if present
        let (theme, keybinds) = load_config();

        Self {
            doc: MlaDocument::default(),
            theme,
            keybinds,
            works_cited_state: WorksCitedModalState::default(),
            citation_state: CitationModalState::default(),
            compliance_state: ComplianceModalState::default(),
            settings_state: SettingsModalState::default(),
            calendar_state: CalendarModalState::default(),
            focus_mode: false,
            vibrancy_dirty: true, // Apply vibrancy on first frame
            notification: None,
        }
    }

    pub fn set_notification(&mut self, msg: impl Into<String>) {
        self.notification = Some((msg.into(), std::time::Instant::now()));
    }

    fn handle_action(&mut self, action: Action) {
        match action {
            Action::NewDocument => {
                self.doc = MlaDocument::new_blank();
                self.set_notification("Created new blank MLA document.");
            }
            Action::OpenDocument => {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("MLA Document (*.mladoc)", &["mladoc"])
                    .add_filter("JSON Document (*.json)", &["json"])
                    .pick_file()
                {
                    match std::fs::read_to_string(&path) {
                        Ok(content) => match serde_json::from_str::<MlaDocument>(&content) {
                            Ok(mut doc) => {
                                doc.file_path = Some(path.to_string_lossy().to_string());
                                doc.is_dirty = false;
                                self.doc = doc;
                                self.set_notification("Document opened successfully.");
                            }
                            Err(e) => {
                                self.set_notification(format!("Error parsing document: {}", e))
                            }
                        },
                        Err(e) => self.set_notification(format!("Error reading file: {}", e)),
                    }
                }
            }
            Action::SaveDocument => {
                let target_path = self.doc.file_path.clone().map(PathBuf::from).or_else(|| {
                    rfd::FileDialog::new()
                        .set_file_name("paper.mladoc")
                        .add_filter("MLA Document (*.mladoc)", &["mladoc"])
                        .save_file()
                });

                if let Some(path) = target_path {
                    match serde_json::to_string_pretty(&self.doc) {
                        Ok(json) => match std::fs::write(&path, json) {
                            Ok(_) => {
                                self.doc.file_path = Some(path.to_string_lossy().to_string());
                                self.doc.is_dirty = false;
                                self.set_notification("Document saved.");
                            }
                            Err(e) => self.set_notification(format!("Error saving file: {}", e)),
                        },
                        Err(e) => self.set_notification(format!("Error serializing: {}", e)),
                    }
                }
            }
            Action::ExportDocx => {
                if let Some(path) = rfd::FileDialog::new()
                    .set_file_name("MLA_Paper.docx")
                    .add_filter("Word Document (*.docx)", &["docx"])
                    .save_file()
                {
                    match export_to_docx(&self.doc, &path) {
                        Ok(_) => {
                            self.set_notification(format!("Exported to Word: {}", path.display()))
                        }
                        Err(e) => self.set_notification(format!("Word export error: {}", e)),
                    }
                }
            }
            Action::ExportHtmlPdf => {
                if let Some(path) = rfd::FileDialog::new()
                    .set_file_name("MLA_Paper_Printable.html")
                    .add_filter("Printable HTML (*.html)", &["html"])
                    .save_file()
                {
                    match export_to_html(&self.doc, &path) {
                        Ok(_) => self.set_notification(format!(
                            "Exported Printable HTML: {}",
                            path.display()
                        )),
                        Err(e) => self.set_notification(format!("HTML export error: {}", e)),
                    }
                }
            }
            Action::AddParagraph => {
                self.doc.ensure_blocks_initialized();
                self.doc.add_paragraph(None);
                self.set_notification("Added new body paragraph.");
            }
            Action::InsertBlockQuote => {
                self.doc.ensure_blocks_initialized();
                self.doc.add_blockquote(None);
                self.set_notification("Inserted MLA block quotation.");
            }
            Action::InsertHeading1 => {
                self.doc.ensure_blocks_initialized();
                self.doc.add_heading(1, None);
                self.set_notification("Inserted Level 1 Heading (Bold).");
            }
            Action::InsertHeading2 => {
                self.doc.ensure_blocks_initialized();
                self.doc.add_heading(2, None);
                self.set_notification("Inserted Level 2 Heading (Italic).");
            }
            Action::InsertCitation => {
                self.citation_state.open(None);
            }
            Action::ManageWorksCited => {
                self.works_cited_state.open_new();
            }
            Action::ConvertToMlaTitleCase => {
                self.doc.title = to_mla_title_case(&self.doc.title);
                self.doc.is_dirty = true;
                self.set_notification("Formatted title to MLA Title Case.");
            }
            Action::OpenPreferences => {
                self.settings_state.is_open = true;
            }
            Action::ToggleComplianceCheck => {
                self.compliance_state.is_open = true;
            }
            Action::ToggleFocusMode => {
                self.focus_mode = !self.focus_mode;
            }
        }
    }
}

impl eframe::App for MlaApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        // Transparent clear color allows OS acrylic / mica / blur to shine through
        [0.0, 0.0, 0.0, 0.0]
    }

    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        // Handle vibrancy application
        if self.vibrancy_dirty {
            if let Some(window) = frame.winit_window() {
                self.theme.apply_vibrancy_to_window(window.as_ref());
                self.vibrancy_dirty = false;
            }
        }

        // Check global custom shortcuts
        let input = ui.input(|i| i.clone());
        for &action in Action::all() {
            if self.keybinds.check_action(action, &input) {
                self.handle_action(action);
                break;
            }
        }

        // If in Zen / Focus Mode, Escape key or F11 unconditionally exits!
        if self.focus_mode {
            let input = ui.input(|i| i.clone());
            if input.key_pressed(egui::Key::Escape) || input.key_pressed(egui::Key::F11) {
                self.focus_mode = false;
                self.set_notification("Exited Zen Mode.");
            }
        }

        // Floating Exit Zen Mode button when in Zen Mode
        if self.focus_mode {
            egui::Area::new(egui::Id::new("zen_mode_floating_exit_pill"))
                .anchor(egui::Align2::RIGHT_TOP, [-24.0, 16.0])
                .order(egui::Order::Foreground)
                .show(ui.ctx(), |ui| {
                    egui::Frame::new()
                        .fill(self.theme.card_fill_color())
                        .stroke(self.theme.page_stroke())
                        .corner_radius(20.0)
                        .inner_margin(egui::Margin::symmetric(14, 8))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                if ui
                                    .button(
                                        egui::RichText::new(format!(
                                            "{} Exit Zen Mode (Esc)",
                                            fonts::icons::FOCUS_MODE
                                        ))
                                        .strong()
                                        .size(12.5)
                                        .color(self.theme.accent_color()),
                                    )
                                    .on_hover_text("Click or press Escape / F11 to exit Zen Mode")
                                    .clicked()
                                {
                                    self.focus_mode = false;
                                    self.set_notification("Exited Zen Mode.");
                                }
                            });
                        });
                });
        }

        // Outer App Container with customizable window opacity & tint
        egui::Frame::new()
            .fill(self.theme.window_fill_color())
            .inner_margin(egui::Margin::symmetric(14, 10))
            .show(ui, |ui| {
                // Top Toolbar (hidden in Focus Mode for total immersion)
                if !self.focus_mode {
                    if let Some(tb_event) =
                        render_toolbar(ui, &self.doc, &self.theme, &self.keybinds, self.focus_mode)
                    {
                        match tb_event {
                            ToolbarEvent::NewDoc => self.handle_action(Action::NewDocument),
                            ToolbarEvent::OpenDoc => self.handle_action(Action::OpenDocument),
                            ToolbarEvent::SaveDoc => self.handle_action(Action::SaveDocument),
                            ToolbarEvent::ExportDocx => self.handle_action(Action::ExportDocx),
                            ToolbarEvent::ExportHtml => self.handle_action(Action::ExportHtmlPdf),
                            ToolbarEvent::ExportText => {
                                if let Some(path) = rfd::FileDialog::new()
                                    .set_file_name("MLA_Paper.txt")
                                    .add_filter("Text File (*.txt)", &["txt"])
                                    .save_file()
                                {
                                    let _ = export_to_text(&self.doc, &path);
                                    self.set_notification("Exported plain text.");
                                }
                            }
                            ToolbarEvent::AddParagraph => self.handle_action(Action::AddParagraph),
                            ToolbarEvent::AddBlockquote => {
                                self.handle_action(Action::InsertBlockQuote)
                            }
                            ToolbarEvent::AddHeading(level) => {
                                if level == 1 {
                                    self.handle_action(Action::InsertHeading1);
                                } else {
                                    self.handle_action(Action::InsertHeading2);
                                }
                            }
                            ToolbarEvent::InsertCitation => {
                                self.handle_action(Action::InsertCitation)
                            }
                            ToolbarEvent::FormatTitleCase => {
                                self.handle_action(Action::ConvertToMlaTitleCase)
                            }
                            ToolbarEvent::OpenWorksCited => {
                                self.handle_action(Action::ManageWorksCited)
                            }
                            ToolbarEvent::OpenCompliance => {
                                self.handle_action(Action::ToggleComplianceCheck)
                            }
                            ToolbarEvent::OpenSettings => {
                                self.handle_action(Action::OpenPreferences)
                            }
                            ToolbarEvent::ToggleFocusMode => {
                                self.handle_action(Action::ToggleFocusMode)
                            }
                        }
                    }
                    ui.add_space(4.0);
                }

                // Temporary notification toast banner
                if let Some((msg, created)) = &self.notification {
                    if created.elapsed().as_secs() < 4 {
                        ui.horizontal(|ui| {
                            ui.colored_label(
                                egui::Color32::from_rgb(100, 200, 255),
                                format!("ℹ {}", msg),
                            );
                        });
                    }
                }

                // Core Editor Canvas
                let editor_action = render_editor_page(
                    ui,
                    &mut self.doc,
                    &self.theme,
                    &self.keybinds,
                    self.focus_mode,
                );

                if let Some(ea) = editor_action {
                    match ea {
                        EditorAction::OpenCitationModal(target) => {
                            self.citation_state.open(target);
                        }
                        EditorAction::OpenWorksCitedModal(maybe_idx) => {
                            if let Some(idx) = maybe_idx {
                                let entry = self.doc.works_cited[idx].clone();
                                self.works_cited_state.open_edit(idx, &entry);
                            } else {
                                self.works_cited_state.open_new();
                            }
                        }
                        EditorAction::OpenCalendar(pos) => {
                            self.calendar_state.open_at(pos, &self.doc.header.date);
                        }
                        EditorAction::TriggerAction(act) => {
                            self.handle_action(act);
                        }
                    }
                }

                // Bottom Status Bar
                if !self.focus_mode {
                    ui.separator();
                    render_status_bar(ui, &self.doc, &self.theme);
                }
            });

        // --- Render Modals ---
        let mut citation_insert = None;
        render_citation_modal(
            ui.ctx(),
            &mut self.citation_state,
            &self.doc.works_cited,
            &self.theme,
            &mut citation_insert,
        );

        if let Some(cite_str) = citation_insert {
            self.doc.ensure_blocks_initialized();
            if let Some(target_idx) = self.citation_state.target_block_index {
                if target_idx < self.doc.blocks.len() {
                    match &mut self.doc.blocks[target_idx] {
                        model::MlaBlock::BlockQuote { citation, .. } => {
                            *citation = cite_str.clone();
                        }
                        model::MlaBlock::Paragraph { text, .. } => {
                            if !text.is_empty() && !text.ends_with(' ') {
                                text.push(' ');
                            }
                            text.push_str(&cite_str);
                        }
                        model::MlaBlock::SectionHeading { text, .. } => {
                            if !text.is_empty() && !text.ends_with(' ') {
                                text.push(' ');
                            }
                            text.push_str(&cite_str);
                        }
                    }
                }
            } else if let Some(last_block) = self.doc.blocks.last_mut() {
                match last_block {
                    model::MlaBlock::BlockQuote { citation, .. } => {
                        *citation = cite_str.clone();
                    }
                    model::MlaBlock::Paragraph { text, .. } => {
                        if !text.is_empty() && !text.ends_with(' ') {
                            text.push(' ');
                        }
                        text.push_str(&cite_str);
                    }
                    model::MlaBlock::SectionHeading { text, .. } => {
                        if !text.is_empty() && !text.ends_with(' ') {
                            text.push(' ');
                        }
                        text.push_str(&cite_str);
                    }
                }
            }
            self.doc.sync_body_from_blocks();
            self.doc.is_dirty = true;
            self.set_notification("Citation inserted.");
        }

        render_works_cited_modal(
            ui.ctx(),
            &mut self.works_cited_state,
            &mut self.doc.works_cited,
            &self.theme,
        );

        render_compliance_modal(
            ui.ctx(),
            &mut self.compliance_state,
            &mut self.doc,
            &self.theme,
        );

        render_settings_modal(
            ui.ctx(),
            &mut self.settings_state,
            &mut self.theme,
            &mut self.keybinds,
            &mut self.vibrancy_dirty,
        );

        render_calendar_popup(
            ui.ctx(),
            &mut self.calendar_state,
            &mut self.doc.header.date,
            &mut self.doc.is_dirty,
            &self.theme,
        );
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct SavedConfig {
    theme: Option<ThemeConfig>,
    keybinds: Option<KeybindConfig>,
}

fn config_path() -> PathBuf {
    // Portable config in current working directory first, fallback to user directory
    let local = Path::new("mla_config.json");
    if local.exists() {
        return local.to_path_buf();
    }
    local.to_path_buf()
}

fn load_config() -> (ThemeConfig, KeybindConfig) {
    let path = config_path();
    if let Ok(content) = std::fs::read_to_string(&path) {
        if let Ok(cfg) = serde_json::from_str::<SavedConfig>(&content) {
            return (
                cfg.theme.unwrap_or_default(),
                cfg.keybinds.unwrap_or_default(),
            );
        }
    }
    (ThemeConfig::default(), KeybindConfig::default())
}
