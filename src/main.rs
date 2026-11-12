#![allow(dead_code)]
#![windows_subsystem = "windows"]
use eframe::egui;
use std::path::PathBuf;

mod export;
mod fonts;
mod history;
mod keybinds;
mod mla_rules;
mod model;
mod theme;
mod ui;
mod writing_editor;

use export::{
    export_to_docx_with_font, export_to_html_with_font, export_to_pdf_with_font, export_to_text,
};
use history::HistoryManager;
use keybinds::{Action, KeybindConfig};
use model::{to_mla_title_case, MlaBlock, MlaDocument};
use theme::ThemeConfig;
use ui::{
    render_calendar_popup, render_citation_modal, render_compliance_modal, render_editor_page,
    render_settings_modal_with_editor, render_status_bar, render_toolbar, render_unsaved_dialog,
    render_works_cited_modal, render_writing_editor_modal, CalendarModalState, CitationModalState,
    ComplianceModalState, EditorAction, SettingsModalState, SettingsTab, StatusBarEvent,
    ToolbarEvent, UnsavedDialogResponse, UnsavedDialogState, WorksCitedModalState,
    WritingEditorModalEvent, WritingEditorModalState,
};
use writing_editor::{EditorSettings, WritingEditor};

const EMBEDDED_APP_ICO: &[u8] = include_bytes!("../assets/scholia-mla.ico");

fn load_app_icon() -> Option<std::sync::Arc<egui::IconData>> {
    // Parse 64x64 32-bit BGRA DIB entry from embedded ICO
    if EMBEDDED_APP_ICO.len() < 6 {
        return None;
    }
    let count = u16::from_le_bytes([EMBEDDED_APP_ICO[4], EMBEDDED_APP_ICO[5]]) as usize;
    let mut chosen_entry = None;
    for i in 0..count {
        let off = 6 + i * 16;
        if off + 16 > EMBEDDED_APP_ICO.len() {
            break;
        }
        let w = match EMBEDDED_APP_ICO[off] {
            0 => 256usize,
            v => v as usize,
        };
        let h = match EMBEDDED_APP_ICO[off + 1] {
            0 => 256usize,
            v => v as usize,
        };
        let bpp = u16::from_le_bytes([EMBEDDED_APP_ICO[off + 6], EMBEDDED_APP_ICO[off + 7]]);
        let size = u32::from_le_bytes([
            EMBEDDED_APP_ICO[off + 8],
            EMBEDDED_APP_ICO[off + 9],
            EMBEDDED_APP_ICO[off + 10],
            EMBEDDED_APP_ICO[off + 11],
        ]) as usize;
        let data_off = u32::from_le_bytes([
            EMBEDDED_APP_ICO[off + 12],
            EMBEDDED_APP_ICO[off + 13],
            EMBEDDED_APP_ICO[off + 14],
            EMBEDDED_APP_ICO[off + 15],
        ]) as usize;
        if bpp == 32 && data_off + size <= EMBEDDED_APP_ICO.len() && size >= 40 + w * h * 4 {
            chosen_entry = Some((w, h, data_off));
            if w == 64 {
                break;
            }
        }
    }
    let (w, h, data_off) = chosen_entry?;
    let pixel_start = data_off + 40;
    let mut rgba = vec![0u8; w * h * 4];
    for y in 0..h {
        let src_row = h - 1 - y;
        for x in 0..w {
            let src_idx = pixel_start + (src_row * w + x) * 4;
            let dst_idx = (y * w + x) * 4;
            rgba[dst_idx] = EMBEDDED_APP_ICO[src_idx + 2]; // R
            rgba[dst_idx + 1] = EMBEDDED_APP_ICO[src_idx + 1]; // G
            rgba[dst_idx + 2] = EMBEDDED_APP_ICO[src_idx]; // B
            rgba[dst_idx + 3] = EMBEDDED_APP_ICO[src_idx + 3]; // A
        }
    }
    Some(std::sync::Arc::new(egui::IconData {
        rgba,
        width: w as u32,
        height: h as u32,
    }))
}

fn main() -> eframe::Result<()> {
    // Windows/Linux: frameless + in-app window controls + custom resize borders.
    // macOS: native traffic lights over the toolbar (no extra title-bar strip).
    let is_macos = cfg!(target_os = "macos");
    let mut viewport = egui::ViewportBuilder::default()
        .with_transparent(true)
        .with_decorations(is_macos)
        .with_fullsize_content_view(is_macos)
        .with_titlebar_shown(false)
        .with_title_shown(false)
        .with_titlebar_buttons_shown(is_macos)
        .with_inner_size([1120.0, 860.0])
        .with_min_inner_size([720.0, 500.0])
        .with_title("Scholia");

    if let Some(icon) = load_app_icon() {
        viewport = viewport.with_icon(icon);
    }

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Scholia",
        native_options,
        Box::new(|cc| {
            fonts::configure_fonts(&cc.egui_ctx);
            let app = MlaApp::new();
            cc.egui_ctx.set_visuals(app.theme.create_egui_visuals());
            Ok(Box::new(app))
        }),
    )
}

struct MlaApp {
    doc: MlaDocument,
    history: HistoryManager,
    theme: ThemeConfig,
    keybinds: KeybindConfig,
    editor_settings: EditorSettings,

    // Modal states
    works_cited_state: WorksCitedModalState,
    citation_state: CitationModalState,
    compliance_state: ComplianceModalState,
    writing_editor_state: WritingEditorModalState,
    settings_state: SettingsModalState,
    calendar_state: CalendarModalState,
    unsaved_changes_state: UnsavedDialogState,

    cached_editor_hash: u64,
    cached_editor_count: usize,
    confirmed_exit: bool,
    focus_mode: bool,
    vibrancy_dirty: bool,
    notification: Option<(String, std::time::Instant)>,
}

impl MlaApp {
    pub fn new() -> Self {
        // Ensure default custom themes exist in themes/ folder
        theme::ensure_sample_themes();
        writing_editor::prewarm_editor();

        // Load saved theme & keybinds & editor settings & undo history limit if present
        let (theme, keybinds, editor_settings, undo_limit) = load_config();
        let mut doc = MlaDocument::default();
        if let Some(arg_path) = std::env::args().nth(1) {
            let path = PathBuf::from(&arg_path);
            if path.exists() && path.is_file() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(mut loaded_doc) = serde_json::from_str::<MlaDocument>(&content) {
                        loaded_doc.file_path = Some(path.to_string_lossy().to_string());
                        loaded_doc.is_dirty = false;
                        doc = loaded_doc;
                    }
                }
            }
        }
        let mut history = HistoryManager::new(&doc);
        history.set_max_depth(undo_limit);

        Self {
            doc,
            history,
            theme,
            keybinds,
            editor_settings,
            works_cited_state: WorksCitedModalState::default(),
            citation_state: CitationModalState::default(),
            compliance_state: ComplianceModalState::default(),
            writing_editor_state: WritingEditorModalState::default(),
            settings_state: SettingsModalState::default(),
            calendar_state: CalendarModalState::default(),
            unsaved_changes_state: UnsavedDialogState::default(),
            cached_editor_hash: 0,
            cached_editor_count: 0,
            confirmed_exit: false,
            focus_mode: false,
            vibrancy_dirty: true, // Apply vibrancy on first frame
            notification: None,
        }
    }

    pub fn set_notification(&mut self, msg: impl Into<String>) {
        self.notification = Some((msg.into(), std::time::Instant::now()));
    }

    pub fn save_current_document(&mut self) -> bool {
        let target_path = self.doc.file_path.clone().map(PathBuf::from).or_else(|| {
            rfd::FileDialog::new()
                .set_file_name(default_document_filename(&self.doc, "mla"))
                .add_filter("MLA Document (*.mla)", &["mla", "mladoc"])
                .save_file()
        });

        if let Some(path) = target_path {
            match serde_json::to_string_pretty(&self.doc) {
                Ok(json) => match std::fs::write(&path, json) {
                    Ok(_) => {
                        self.doc.file_path = Some(path.to_string_lossy().to_string());
                        self.doc.is_dirty = false;
                        self.set_notification("Document saved.");
                        true
                    }
                    Err(e) => {
                        self.set_notification(format!("Error saving file: {}", e));
                        false
                    }
                },
                Err(e) => {
                    self.set_notification(format!("Error serializing: {}", e));
                    false
                }
            }
        } else {
            false
        }
    }

    fn handle_action(&mut self, action: Action, ctx: &egui::Context) {
        match action {
            Action::NewDocument => {
                self.doc = MlaDocument::new_blank();
                self.history.reset(&self.doc);
                self.set_notification("Created new blank MLA document.");
            }
            Action::OpenDocument => {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("MLA Document (*.mla)", &["mla", "mladoc"])
                    .add_filter("JSON Document (*.json)", &["json"])
                    .pick_file()
                {
                    match std::fs::read_to_string(&path) {
                        Ok(content) => match serde_json::from_str::<MlaDocument>(&content) {
                            Ok(mut doc) => {
                                doc.file_path = Some(path.to_string_lossy().to_string());
                                doc.is_dirty = false;
                                self.doc = doc;
                                self.history.reset(&self.doc);
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
                self.save_current_document();
            }
            Action::ExportDocx => {
                if let Some(path) = rfd::FileDialog::new()
                    .set_file_name(default_document_filename(&self.doc, "docx"))
                    .add_filter("Word Document (*.docx)", &["docx"])
                    .save_file()
                {
                    match export_to_docx_with_font(&self.doc, &path, self.theme.font_choice) {
                        Ok(_) => {
                            self.set_notification(format!("Exported to Word: {}", path.display()))
                        }
                        Err(e) => self.set_notification(format!("Word export error: {}", e)),
                    }
                }
            }
            Action::ExportHtmlPdf => {
                if let Some(path) = rfd::FileDialog::new()
                    .set_file_name(default_document_filename(&self.doc, "pdf"))
                    .add_filter("PDF Document (*.pdf)", &["pdf"])
                    .save_file()
                {
                    match export_to_pdf_with_font(&self.doc, &path, self.theme.font_choice) {
                        Ok(_) => self.set_notification(format!("Exported PDF: {}", path.display())),
                        Err(e) => self.set_notification(format!("PDF export error: {}", e)),
                    }
                }
            }
            Action::Undo => {
                self.handle_undo(ctx);
            }
            Action::Redo => {
                self.handle_redo(ctx);
            }
            Action::AddParagraph => {
                let cursor = self.get_active_block_cursor(ctx);
                self.history.record_discrete_action(&self.doc, cursor);
                self.doc.ensure_blocks_initialized();
                let current_idx = if self.doc.blocks.is_empty() {
                    None
                } else {
                    Some(self.doc.active_block_idx.min(self.doc.blocks.len() - 1))
                };
                let new_idx = self.doc.add_paragraph(current_idx);
                self.doc.active_block_idx = new_idx;
                self.doc.requested_focus_block_idx = Some(new_idx);
                self.set_notification("Added new body paragraph.");
            }
            Action::InsertBlockQuote => {
                let cursor = self.get_active_block_cursor(ctx);
                self.history.record_discrete_action(&self.doc, cursor);
                self.doc.ensure_blocks_initialized();
                let current_idx = if self.doc.blocks.is_empty() {
                    None
                } else {
                    Some(self.doc.active_block_idx.min(self.doc.blocks.len() - 1))
                };
                let new_idx = self.doc.add_blockquote(current_idx);
                self.doc.active_block_idx = new_idx;
                self.doc.requested_focus_block_idx = Some(new_idx);
                self.set_notification("Inserted MLA block quotation.");
            }
            Action::InsertHeading1 => {
                let cursor = self.get_active_block_cursor(ctx);
                self.history.record_discrete_action(&self.doc, cursor);
                self.doc.ensure_blocks_initialized();
                let current_idx = if self.doc.blocks.is_empty() {
                    None
                } else {
                    Some(self.doc.active_block_idx.min(self.doc.blocks.len() - 1))
                };
                let new_idx = self.doc.add_heading(1, current_idx);
                self.doc.active_block_idx = new_idx;
                self.doc.requested_focus_block_idx = Some(new_idx);
                self.set_notification("Inserted Level 1 Heading (Bold).");
            }
            Action::InsertHeading2 => {
                let cursor = self.get_active_block_cursor(ctx);
                self.history.record_discrete_action(&self.doc, cursor);
                self.doc.ensure_blocks_initialized();
                let current_idx = if self.doc.blocks.is_empty() {
                    None
                } else {
                    Some(self.doc.active_block_idx.min(self.doc.blocks.len() - 1))
                };
                let new_idx = self.doc.add_heading(2, current_idx);
                self.doc.active_block_idx = new_idx;
                self.doc.requested_focus_block_idx = Some(new_idx);
                self.set_notification("Inserted Level 2 Heading (Italic).");
            }
            Action::InsertHeading3 => {
                let cursor = self.get_active_block_cursor(ctx);
                self.history.record_discrete_action(&self.doc, cursor);
                self.doc.ensure_blocks_initialized();
                let current_idx = if self.doc.blocks.is_empty() {
                    None
                } else {
                    Some(self.doc.active_block_idx.min(self.doc.blocks.len() - 1))
                };
                let new_idx = self.doc.add_heading(3, current_idx);
                self.doc.active_block_idx = new_idx;
                self.doc.requested_focus_block_idx = Some(new_idx);
                self.set_notification("Inserted Level 3 Heading (Centered).");
            }
            Action::InsertCitation => {
                let target = if self.doc.blocks.is_empty() {
                    None
                } else {
                    Some(self.doc.active_block_idx.min(self.doc.blocks.len() - 1))
                };
                self.citation_state.open(target);
            }
            Action::DeleteBlock => {
                if self.doc.blocks.len() > 1 {
                    let cursor = self.get_active_block_cursor(ctx);
                    self.history.record_discrete_action(&self.doc, cursor);
                    let idx = self.doc.active_block_idx.min(self.doc.blocks.len() - 1);
                    let prev_idx = if idx > 0 { idx - 1 } else { 0 };
                    self.doc.remove_block(idx);
                    self.doc.active_block_idx = prev_idx;
                    self.doc.requested_focus_block_idx = Some(prev_idx);
                    self.set_notification("Deleted block.");
                } else {
                    self.set_notification("Cannot delete the only paragraph.");
                }
            }
            Action::MoveBlockUp => {
                if self.doc.active_block_idx > 0 {
                    let cursor = self.get_active_block_cursor(ctx);
                    self.history.record_discrete_action(&self.doc, cursor);
                    self.doc.move_block_up(self.doc.active_block_idx);
                    self.doc.active_block_idx -= 1;
                    self.doc.requested_focus_block_idx = Some(self.doc.active_block_idx);
                    self.set_notification("Moved block up.");
                }
            }
            Action::MoveBlockDown => {
                if self.doc.active_block_idx + 1 < self.doc.blocks.len() {
                    let cursor = self.get_active_block_cursor(ctx);
                    self.history.record_discrete_action(&self.doc, cursor);
                    self.doc.move_block_down(self.doc.active_block_idx);
                    self.doc.active_block_idx += 1;
                    self.doc.requested_focus_block_idx = Some(self.doc.active_block_idx);
                    self.set_notification("Moved block down.");
                }
            }
            Action::ManageWorksCited => {
                self.works_cited_state.open_new();
            }
            Action::AddFootnote => {
                let cursor = self.get_active_block_cursor(ctx);
                self.history.record_discrete_action(&self.doc, cursor);
                let note_idx = self.doc.add_explanatory_note(String::new());
                let sup = model::num_to_superscript(note_idx);
                self.set_notification(format!("Added Note {} linked to active paragraph.", sup));
            }
            Action::ConvertToMlaTitleCase => {
                let cursor = self.get_active_block_cursor(ctx);
                self.history.record_discrete_action(&self.doc, cursor);
                self.doc.title = to_mla_title_case(&self.doc.title);
                self.doc.is_dirty = true;
                self.set_notification("Formatted title to MLA Title Case.");
            }
            Action::MoveSentenceLeft => {
                self.handle_move_sentence(ctx, true);
            }
            Action::MoveSentenceRight => {
                self.handle_move_sentence(ctx, false);
            }
            Action::OpenPreferences => {
                self.settings_state.is_open = true;
            }
            Action::ToggleComplianceCheck => {
                self.compliance_state.is_open = true;
            }
            Action::ToggleFocusMode => {
                self.focus_mode = !self.focus_mode;
                if self.focus_mode {
                    self.set_notification("Entered Zen Mode (Press Esc or F11 to exit).");
                } else {
                    self.set_notification("Exited Zen Mode.");
                }
            }
        }
    }

    fn handle_move_sentence(&mut self, ctx: &egui::Context, direction_left: bool) {
        if self.doc.blocks.is_empty() {
            return;
        }
        let b_idx = self.doc.active_block_idx.min(self.doc.blocks.len() - 1);
        let block = &self.doc.blocks[b_idx];
        let target_id = match block {
            MlaBlock::Paragraph { id, .. } => egui::Id::new("p_block").with(id),
            MlaBlock::BlockQuote { id, .. } => egui::Id::new("bq_block").with(id),
            MlaBlock::SectionHeading { id, .. } => egui::Id::new("h_block").with(id),
        };

        let cursor_idx = self.get_active_block_cursor(ctx);
        self.history.record_discrete_action(&self.doc, cursor_idx);

        match self
            .doc
            .move_sentence_in_active_block(cursor_idx, direction_left)
        {
            Ok(new_cursor) => {
                let mut state =
                    egui::text_edit::TextEditState::load(ctx, target_id).unwrap_or_default();
                state
                    .cursor
                    .set_char_range(Some(egui::text::CCursorRange::one(
                        egui::text::CCursor::new(new_cursor),
                    )));
                state.store(ctx, target_id);
                ctx.memory_mut(|m| m.request_focus(target_id));
                let dir_str = if direction_left { "left" } else { "right" };
                self.set_notification(format!("Moved sentence {dir_str}."));
            }
            Err(msg) => {
                self.set_notification(msg);
            }
        }
    }

    fn get_active_block_cursor(&self, ctx: &egui::Context) -> usize {
        let b_idx = self
            .doc
            .active_block_idx
            .min(self.doc.blocks.len().saturating_sub(1));
        self.doc
            .blocks
            .get(b_idx)
            .and_then(|block| {
                let target_id = match block {
                    MlaBlock::Paragraph { id, .. } => egui::Id::new("p_block").with(id),
                    MlaBlock::BlockQuote { id, .. } => egui::Id::new("bq_block").with(id),
                    MlaBlock::SectionHeading { id, .. } => egui::Id::new("h_block").with(id),
                };
                egui::text_edit::TextEditState::load(ctx, target_id)
                    .and_then(|s| s.cursor.char_range())
                    .map(|r| r.primary.index.0)
            })
            .unwrap_or(0)
    }

    fn consume_undo_redo_keys(ctx: &egui::Context) {
        ctx.input_mut(|i| {
            i.events.retain(|e| match e {
                egui::Event::Key { key, modifiers, .. } => {
                    let ctrl = modifiers.command || modifiers.ctrl;
                    !((*key == egui::Key::Z || *key == egui::Key::Y) && ctrl)
                }
                _ => true,
            });
        });
    }

    fn apply_snapshot(&mut self, snapshot: crate::history::Snapshot, ctx: &egui::Context) {
        self.doc = snapshot.doc;
        self.doc.is_dirty = true;
        self.doc.sync_body_from_blocks();

        let b_idx = self
            .doc
            .active_block_idx
            .min(self.doc.blocks.len().saturating_sub(1));
        if let Some(block) = self.doc.blocks.get(b_idx) {
            let target_id = match block {
                MlaBlock::Paragraph { id, .. } => egui::Id::new("p_block").with(id),
                MlaBlock::BlockQuote { id, .. } => egui::Id::new("bq_block").with(id),
                MlaBlock::SectionHeading { id, .. } => egui::Id::new("h_block").with(id),
            };
            let mut state =
                egui::text_edit::TextEditState::load(ctx, target_id).unwrap_or_default();
            let safe_cursor = snapshot.cursor_pos.min(block.text().chars().count());
            state
                .cursor
                .set_char_range(Some(egui::text::CCursorRange::one(
                    egui::text::CCursor::new(safe_cursor),
                )));
            state.store(ctx, target_id);
            ctx.memory_mut(|m| m.request_focus(target_id));
        }
    }

    fn handle_undo(&mut self, ctx: &egui::Context) {
        let cursor = self.get_active_block_cursor(ctx);
        if let Some(snapshot) = self.history.undo(&self.doc, cursor) {
            self.apply_snapshot(snapshot, ctx);
            self.set_notification("Undo");
        } else {
            self.set_notification("Nothing to undo.");
        }
        Self::consume_undo_redo_keys(ctx);
    }

    fn handle_redo(&mut self, ctx: &egui::Context) {
        let cursor = self.get_active_block_cursor(ctx);
        if let Some(snapshot) = self.history.redo(&self.doc, cursor) {
            self.apply_snapshot(snapshot, ctx);
            self.set_notification("Redo");
        } else {
            self.set_notification("Nothing to redo.");
        }
        Self::consume_undo_redo_keys(ctx);
    }
}

impl eframe::App for MlaApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        // Transparent clear color lets OS acrylic / mica / vibrancy / compositor transparency show through
        [0.0, 0.0, 0.0, 0.0]
    }

    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        // Apply theme visuals so all buttons, dialogs, and widgets are transparent
        let theme_visuals = self.theme.create_egui_visuals();
        ui.ctx().set_visuals(theme_visuals.clone());
        ui.style_mut().visuals = theme_visuals;

        // Ensure no 1px white line / border is drawn on borderless window
        ui.style_mut().visuals.window_stroke = egui::Stroke::NONE;

        // Fill the entire window bounds seamlessly with the theme's window fill color
        let screen_rect = ui.max_rect();
        ui.painter()
            .rect_filled(screen_rect, 0.0, self.theme.window_fill_color());

        // Interactive frameless window edge/corner resize handles on Windows and Linux
        #[cfg(not(target_os = "macos"))]
        {
            let is_max = ui.input(|i| i.viewport().maximized.unwrap_or(false));
            if !is_max && !self.focus_mode {
                handle_frameless_resize(ui.ctx(), screen_rect);
            }
        }

        // Handle vibrancy application
        if self.vibrancy_dirty {
            if let Some(window) = frame.winit_window() {
                self.theme.apply_vibrancy_to_window(window.as_ref());
                self.vibrancy_dirty = false;
            }
        }

        // Intercept close requests across all mechanisms (OS close, Alt+F4, Ctrl+Q, Toolbar close button)
        let mut close_requested = ui.ctx().input(|i| i.viewport().close_requested());

        // Keyboard shortcuts to close the application (Alt+F4 or Ctrl+Q)
        ui.ctx().input(|i| {
            if (i.modifiers.alt && i.key_pressed(egui::Key::F4))
                || (i.modifiers.command && i.key_pressed(egui::Key::Q))
            {
                close_requested = true;
            }
        });

        // Check global custom shortcuts
        let input = ui.input(|i| i.clone());
        for &action in Action::all() {
            if self.keybinds.check_action(action, &input) {
                self.handle_action(action, ui.ctx());
                break;
            }
        }

        // Handle dropped files (drag & drop opening)
        let dropped = ui.ctx().input(|i| i.raw.dropped_files.clone());
        for file in dropped {
            let path = file.path();
            if path.exists() && path.is_file() {
                if let Ok(content) = std::fs::read_to_string(path) {
                    if let Ok(mut loaded_doc) = serde_json::from_str::<MlaDocument>(&content) {
                        loaded_doc.file_path = Some(path.to_string_lossy().to_string());
                        loaded_doc.is_dirty = false;
                        self.doc = loaded_doc;
                        self.history.reset(&self.doc);
                        self.set_notification("Document opened from file.");
                    }
                }
            }
        }

        // If in Zen / Focus Mode, only Escape key exits!
        if self.focus_mode {
            let input = ui.input(|i| i.clone());
            if input.key_pressed(egui::Key::Escape) {
                self.focus_mode = false;
                self.set_notification("Exited Zen Mode.");
            }
        }

        // Outer App Container — 8px top/bottom padding for visual breathing room
        egui::Frame::NONE
            .fill(self.theme.window_fill_color())
            .inner_margin(egui::Margin {
                left: 12,
                right: 12,
                top: 8,
                bottom: 8,
            })
            .show(ui, |ui| {
                // Top Toolbar (hidden in Focus Mode for total immersion)
                if !self.focus_mode {
                    if let Some(tb_event) =
                        render_toolbar(ui, &self.doc, &self.theme, &self.keybinds, self.focus_mode)
                    {
                        match tb_event {
                            ToolbarEvent::NewDoc => {
                                self.handle_action(Action::NewDocument, ui.ctx())
                            }
                            ToolbarEvent::OpenDoc => {
                                self.handle_action(Action::OpenDocument, ui.ctx())
                            }
                            ToolbarEvent::SaveDoc => {
                                self.handle_action(Action::SaveDocument, ui.ctx())
                            }
                            ToolbarEvent::Undo => self.handle_action(Action::Undo, ui.ctx()),
                            ToolbarEvent::Redo => self.handle_action(Action::Redo, ui.ctx()),
                            ToolbarEvent::ExportDocx => {
                                self.handle_action(Action::ExportDocx, ui.ctx())
                            }
                            ToolbarEvent::ExportHtml => {
                                self.handle_action(Action::ExportHtmlPdf, ui.ctx())
                            }
                            ToolbarEvent::ExportHtmlFile => {
                                if let Some(path) = rfd::FileDialog::new()
                                    .set_file_name(default_document_filename(&self.doc, "html"))
                                    .add_filter("HTML Document (*.html)", &["html", "htm"])
                                    .save_file()
                                {
                                    match export_to_html_with_font(
                                        &self.doc,
                                        &path,
                                        self.theme.font_choice,
                                    ) {
                                        Ok(_) => self.set_notification(format!(
                                            "Exported HTML: {}",
                                            path.display()
                                        )),
                                        Err(e) => self
                                            .set_notification(format!("HTML export error: {}", e)),
                                    }
                                }
                            }
                            ToolbarEvent::ExportText => {
                                if let Some(path) = rfd::FileDialog::new()
                                    .set_file_name(default_document_filename(&self.doc, "txt"))
                                    .add_filter("Text File (*.txt)", &["txt"])
                                    .save_file()
                                {
                                    match export_to_text(&self.doc, &path) {
                                        Ok(_) => self.set_notification(format!(
                                            "Exported plain text: {}",
                                            path.display()
                                        )),
                                        Err(e) => self
                                            .set_notification(format!("Text export error: {}", e)),
                                    }
                                }
                            }
                            ToolbarEvent::AddBlockquote => {
                                self.handle_action(Action::InsertBlockQuote, ui.ctx())
                            }
                            ToolbarEvent::AddHeading(level) => {
                                if level == 1 {
                                    self.handle_action(Action::InsertHeading1, ui.ctx());
                                } else if level == 2 {
                                    self.handle_action(Action::InsertHeading2, ui.ctx());
                                } else {
                                    self.handle_action(Action::InsertHeading3, ui.ctx());
                                }
                            }
                            ToolbarEvent::OpenWorksCited => {
                                self.handle_action(Action::ManageWorksCited, ui.ctx())
                            }
                            ToolbarEvent::AddFootnote => {
                                self.handle_action(Action::AddFootnote, ui.ctx())
                            }
                            ToolbarEvent::CloseApp => {
                                close_requested = true;
                            }
                        }
                    }
                }

                // Handle close request confirmation popup
                if close_requested {
                    if self.doc.is_dirty && !self.confirmed_exit {
                        ui.ctx()
                            .send_viewport_cmd(egui::ViewportCommand::CancelClose);
                        self.unsaved_changes_state.is_open = true;
                    } else {
                        save_config(
                            &self.theme,
                            &self.keybinds,
                            &self.editor_settings,
                            self.history.max_depth,
                        );
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
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
                            self.handle_action(act, ui.ctx());
                        }
                        EditorAction::ShowNotification(msg) => {
                            self.set_notification(msg);
                        }
                    }
                }

                // Bottom Status Bar (no grey separator line, flush to bottom)
                if !self.focus_mode {
                    let current_hash = compute_editor_state_hash(&self.doc, &self.editor_settings);
                    if current_hash != self.cached_editor_hash {
                        self.cached_editor_count =
                            WritingEditor::inspect(&self.doc, &self.editor_settings)
                                .total_suggestions();
                        self.cached_editor_hash = current_hash;
                    }

                    match render_status_bar(ui, &self.doc, &self.theme, self.cached_editor_count) {
                        Some(StatusBarEvent::OpenCompliance) => {
                            self.compliance_state.is_open = true;
                        }
                        Some(StatusBarEvent::OpenWritingEditor) => {
                            self.writing_editor_state.is_open = true;
                        }
                        None => {}
                    }
                }
            });

        // Floating toast notification overlay in bottom right just above the footer
        if let Some((msg, created)) = &self.notification {
            if created.elapsed().as_secs() < 4 {
                egui::Area::new(egui::Id::new("toast_notification_overlay"))
                    .anchor(egui::Align2::RIGHT_BOTTOM, [-20.0, -42.0])
                    .order(egui::Order::Foreground)
                    .show(ui.ctx(), |ui| {
                        egui::Frame::new()
                            .fill(self.theme.card_fill_color())
                            .stroke(egui::Stroke::new(1.0, self.theme.accent_color()))
                            .corner_radius(8.0)
                            .shadow(egui::Shadow {
                                offset: [0, 3],
                                blur: 8,
                                spread: 0,
                                color: egui::Color32::from_black_alpha(80),
                            })
                            .inner_margin(egui::Margin::symmetric(14, 8))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new(format!("ℹ  {}", msg))
                                            .size(12.5)
                                            .color(self.theme.text_color()),
                                    );
                                });
                            });
                    });
            }
        }

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
            let cursor = self.get_active_block_cursor(ui.ctx());
            self.history.record_discrete_action(&self.doc, cursor);
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

        let pre_wc = if self.works_cited_state.is_open {
            Some(self.doc.works_cited.clone())
        } else {
            None
        };

        render_works_cited_modal(
            ui.ctx(),
            &mut self.works_cited_state,
            &mut self.doc.works_cited,
            &self.theme,
        );

        if let Some(old_wc) = pre_wc {
            if old_wc != self.doc.works_cited {
                let cursor = self.get_active_block_cursor(ui.ctx());
                let mut prev_doc = self.doc.clone();
                prev_doc.works_cited = old_wc;
                self.history.record_discrete_action(&prev_doc, cursor);
                self.doc.is_dirty = true;
            }
        }

        let pre_comp_doc = if self.compliance_state.is_open {
            Some(self.doc.clone())
        } else {
            None
        };

        render_compliance_modal(
            ui.ctx(),
            &mut self.compliance_state,
            &mut self.doc,
            &self.theme,
        );

        if let Some(old_doc) = pre_comp_doc {
            if !old_doc.content_equals(&self.doc) {
                let cursor = self.get_active_block_cursor(ui.ctx());
                self.history.record_discrete_action(&old_doc, cursor);
            }
        }

        let pre_editor_doc = if self.writing_editor_state.is_open {
            Some(self.doc.clone())
        } else {
            None
        };

        if let Some(ev) = render_writing_editor_modal(
            ui.ctx(),
            &mut self.writing_editor_state,
            &mut self.doc,
            &mut self.editor_settings,
            &self.theme,
        ) {
            match ev {
                WritingEditorModalEvent::OpenEditorSettings => {
                    self.settings_state.selected_tab = SettingsTab::EditorSettings;
                    self.settings_state.is_open = true;
                }
                WritingEditorModalEvent::DictionaryModified => {
                    save_config(
                        &self.theme,
                        &self.keybinds,
                        &self.editor_settings,
                        self.history.max_depth,
                    );
                    self.set_notification("Added word to custom dictionary.");
                }
            }
        }

        if let Some(old_doc) = pre_editor_doc {
            if !old_doc.content_equals(&self.doc) {
                let cursor = self.get_active_block_cursor(ui.ctx());
                self.history.record_discrete_action(&old_doc, cursor);
            }
        }

        let was_settings_open = self.settings_state.is_open;
        render_settings_modal_with_editor(
            ui.ctx(),
            &mut self.settings_state,
            &mut self.theme,
            &mut self.keybinds,
            &mut self.editor_settings,
            &mut self.history.max_depth,
            &mut self.vibrancy_dirty,
        );
        if was_settings_open && !self.settings_state.is_open {
            save_config(
                &self.theme,
                &self.keybinds,
                &self.editor_settings,
                self.history.max_depth,
            );
        }

        render_calendar_popup(
            ui.ctx(),
            &mut self.calendar_state,
            &mut self.doc.header.date,
            &mut self.doc.is_dirty,
            &self.theme,
        );

        let unsaved_resp = render_unsaved_dialog(
            ui.ctx(),
            &mut self.unsaved_changes_state,
            &self.doc,
            &self.theme,
        );
        match unsaved_resp {
            UnsavedDialogResponse::Save => {
                if self.save_current_document() {
                    self.confirmed_exit = true;
                    save_config(
                        &self.theme,
                        &self.keybinds,
                        &self.editor_settings,
                        self.history.max_depth,
                    );
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
            UnsavedDialogResponse::Discard => {
                self.confirmed_exit = true;
                save_config(
                    &self.theme,
                    &self.keybinds,
                    &self.editor_settings,
                    self.history.max_depth,
                );
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
            UnsavedDialogResponse::Cancel | UnsavedDialogResponse::None => {}
        }

        // Commit continuous typing / word groups at end of frame
        let cursor = self.get_active_block_cursor(ui.ctx());
        self.history.on_frame_end(&self.doc, cursor);
    }
}

fn compute_editor_state_hash(doc: &MlaDocument, settings: &EditorSettings) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    doc.title.hash(&mut hasher);
    for b in &doc.blocks {
        b.id().hash(&mut hasher);
        b.text().hash(&mut hasher);
    }
    for n in &doc.notes {
        n.text.hash(&mut hasher);
    }
    for w in &doc.works_cited {
        w.author.hash(&mut hasher);
    }
    doc.header.student_name.hash(&mut hasher);
    doc.header.instructor_name.hash(&mut hasher);
    doc.header.running_header_last_name.hash(&mut hasher);
    settings.language.hash(&mut hasher);
    settings.enable_spell_check.hash(&mut hasher);
    settings.enable_grammar_check.hash(&mut hasher);
    settings.enable_style_check.hash(&mut hasher);
    settings.flag_contractions.hash(&mut hasher);
    settings.ignore_all_caps.hash(&mut hasher);
    settings.ignore_words_with_numbers.hash(&mut hasher);
    settings.custom_dictionary.hash(&mut hasher);
    settings.ignored_issues.len().hash(&mut hasher);
    settings.ignored_words_session.len().hash(&mut hasher);
    hasher.finish()
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct SavedConfig {
    theme: Option<ThemeConfig>,
    keybinds: Option<KeybindConfig>,
    editor_settings: Option<EditorSettings>,
    undo_limit: Option<usize>,
}

fn config_path() -> PathBuf {
    theme::app_data_dir().join("mla_config.json")
}

fn save_config(
    theme: &ThemeConfig,
    keybinds: &KeybindConfig,
    editor_settings: &EditorSettings,
    undo_limit: usize,
) {
    let path = config_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let cfg = SavedConfig {
        theme: Some(theme.clone()),
        keybinds: Some(keybinds.clone()),
        editor_settings: Some(editor_settings.clone()),
        undo_limit: Some(undo_limit.clamp(16, 8192)),
    };
    if let Ok(json) = serde_json::to_string_pretty(&cfg) {
        let _ = std::fs::write(&path, json);
    }
}

fn default_document_filename(doc: &MlaDocument, extension: &str) -> String {
    let title = doc
        .title
        .trim()
        .chars()
        .map(|ch| {
            if matches!(ch, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|') {
                '_'
            } else {
                ch
            }
        })
        .collect::<String>();
    let stem = if title.trim().is_empty() {
        "Untitled MLA Document".to_string()
    } else {
        title.trim().to_string()
    };
    format!("{stem}.{extension}")
}

fn load_config() -> (ThemeConfig, KeybindConfig, EditorSettings, usize) {
    let path = config_path();
    if let Ok(content) = std::fs::read_to_string(&path) {
        if let Ok(cfg) = serde_json::from_str::<SavedConfig>(&content) {
            return (
                cfg.theme.unwrap_or_default(),
                cfg.keybinds.unwrap_or_default(),
                cfg.editor_settings.unwrap_or_default(),
                cfg.undo_limit.unwrap_or(512).clamp(16, 8192),
            );
        }
    }
    (
        ThemeConfig::default(),
        KeybindConfig::default(),
        EditorSettings::default(),
        512,
    )
}

#[cfg(not(target_os = "macos"))]
fn handle_frameless_resize(ctx: &egui::Context, screen_rect: egui::Rect) {
    if ctx.egui_is_using_pointer() {
        return;
    }
    let Some(pos) = ctx.input(|i| i.pointer.hover_pos()) else {
        return;
    };

    let edge = 5.0;
    let corner = 10.0;

    let left = pos.x - screen_rect.min.x <= edge;
    let right = screen_rect.max.x - pos.x <= edge;
    let top = pos.y - screen_rect.min.y <= edge;
    let bottom = screen_rect.max.y - pos.y <= edge;

    if !left && !right && !top && !bottom {
        return;
    }

    let left_c = pos.x - screen_rect.min.x <= corner;
    let right_c = screen_rect.max.x - pos.x <= corner;
    let top_c = pos.y - screen_rect.min.y <= corner;
    let bottom_c = screen_rect.max.y - pos.y <= corner;

    let dir_and_cursor = if top_c && left_c {
        Some((
            egui::ResizeDirection::NorthWest,
            egui::CursorIcon::ResizeNorthWest,
        ))
    } else if top_c && right_c {
        Some((
            egui::ResizeDirection::NorthEast,
            egui::CursorIcon::ResizeNorthEast,
        ))
    } else if bottom_c && left_c {
        Some((
            egui::ResizeDirection::SouthWest,
            egui::CursorIcon::ResizeSouthWest,
        ))
    } else if bottom_c && right_c {
        Some((
            egui::ResizeDirection::SouthEast,
            egui::CursorIcon::ResizeSouthEast,
        ))
    } else if left {
        Some((egui::ResizeDirection::West, egui::CursorIcon::ResizeWest))
    } else if right {
        Some((egui::ResizeDirection::East, egui::CursorIcon::ResizeEast))
    } else if top {
        Some((egui::ResizeDirection::North, egui::CursorIcon::ResizeNorth))
    } else if bottom {
        Some((egui::ResizeDirection::South, egui::CursorIcon::ResizeSouth))
    } else {
        None
    };

    if let Some((dir, cursor)) = dir_and_cursor {
        ctx.set_cursor_icon(cursor);
        if ctx.input(|i| i.pointer.primary_pressed()) {
            ctx.send_viewport_cmd(egui::ViewportCommand::BeginResize(dir));
        }
    }
}
