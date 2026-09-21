use crate::fonts::icons;
use crate::keybinds::{Action, KeybindConfig};
use crate::model::MlaDocument;
use crate::theme::ThemeConfig;
use egui::{RichText, Ui};

pub enum ToolbarEvent {
    NewDoc,
    OpenDoc,
    SaveDoc,
    Undo,
    Redo,
    ExportDocx,
    ExportHtml,
    ExportText,
    AddBlockquote,
    AddHeading(u8),
    OpenWorksCited,
    AddFootnote,
    CloseApp,
}

pub fn render_toolbar(
    ui: &mut Ui,
    doc: &MlaDocument,
    theme: &ThemeConfig,
    keybinds: &KeybindConfig,
    _focus_mode: bool,
) -> Option<ToolbarEvent> {
    let mut event = None;
    let text_col = theme.text_color();
    let accent_col = theme.accent_color();

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(6.0, 6.0);

        // Brand / Title
        let brand_resp = ui.label(
            RichText::new("Scholia")
                .strong()
                .size(15.0)
                .color(accent_col),
        );
        let brand_drag = ui.interact(brand_resp.rect, ui.id().with("brand_drag"), egui::Sense::click_and_drag());
        if brand_drag.drag_started() {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
        }
        if brand_drag.double_clicked() {
            let is_max = ui.input(|i| i.viewport().maximized.unwrap_or(false));
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Maximized(!is_max));
        }

        ui.separator();

        // --- File Menu Actions ---
        let new_sc = keybinds.get_shortcut(Action::NewDocument).display_string();
        if ui
            .button(RichText::new(format!("{} New", icons::FILE_NEW)).color(text_col))
            .on_hover_text(format!("Create new document ({})", new_sc))
            .clicked()
        {
            event = Some(ToolbarEvent::NewDoc);
        }

        let open_sc = keybinds.get_shortcut(Action::OpenDocument).display_string();
        if ui
            .button(RichText::new(format!("{} Open", icons::FOLDER_OPEN)).color(text_col))
            .on_hover_text(format!("Open .mla file ({})", open_sc))
            .clicked()
        {
            event = Some(ToolbarEvent::OpenDoc);
        }

        let save_sc = keybinds.get_shortcut(Action::SaveDocument).display_string();
        if ui
            .button(RichText::new(format!("{} Save", icons::SAVE)).color(text_col))
            .on_hover_text(format!("Save document ({})", save_sc))
            .clicked()
        {
            event = Some(ToolbarEvent::SaveDoc);
        }

        let undo_sc = keybinds.get_shortcut(Action::Undo).display_string();
        if ui
            .button(RichText::new(format!("{} Undo", icons::UNDO)).color(text_col))
            .on_hover_text(format!("Undo last action ({})", undo_sc))
            .clicked()
        {
            event = Some(ToolbarEvent::Undo);
        }

        let redo_sc = keybinds.get_shortcut(Action::Redo).display_string();
        if ui
            .button(RichText::new(format!("{} Redo", icons::REDO)).color(text_col))
            .on_hover_text(format!("Redo action ({})", redo_sc))
            .clicked()
        {
            event = Some(ToolbarEvent::Redo);
        }

        // Export dropdown / buttons (shortcuts removed as requested)
        egui::ComboBox::from_id_salt("export_cb")
            .selected_text(
                RichText::new(format!("{} Export", icons::WORD_DOCX))
                    .strong()
                    .color(accent_col),
            )
            .show_ui(ui, |ui| {
                if ui
                    .button(format!("{} Word Document (.docx)", icons::WORD_DOCX))
                    .clicked()
                {
                    event = Some(ToolbarEvent::ExportDocx);
                }
                if ui
                    .button(format!("{} Export PDF (.pdf)", icons::HTML_PDF))
                    .clicked()
                {
                    event = Some(ToolbarEvent::ExportHtml);
                }
                if ui
                    .button(format!("{} Plain Text / Markdown (.txt)", icons::TEXT))
                    .clicked()
                {
                    event = Some(ToolbarEvent::ExportText);
                }
            });

        ui.separator();

        // --- MLA Structure Inserters ---
        let bq_sc = keybinds
            .get_shortcut(Action::InsertBlockQuote)
            .display_string();
        if ui
            .button(RichText::new(format!("{} Block Quote", icons::QUOTE)).color(text_col))
            .on_hover_text(format!(
                "Insert 0.5\" indented block quotation for >4 lines of prose ({})",
                bq_sc
            ))
            .clicked()
        {
            event = Some(ToolbarEvent::AddBlockquote);
        }

        let h1_sc = keybinds
            .get_shortcut(Action::InsertHeading1)
            .display_string();
        if ui
            .button(RichText::new(format!("{} H1", icons::HEADING)).color(text_col))
            .on_hover_text(format!("Insert MLA Level 1 Section Heading (Bold Flush Left) ({})", h1_sc))
            .clicked()
        {
            event = Some(ToolbarEvent::AddHeading(1));
        }

        let h2_sc = keybinds
            .get_shortcut(Action::InsertHeading2)
            .display_string();
        if ui
            .button(RichText::new(format!("{} H2", icons::HEADING)).color(text_col))
            .on_hover_text(format!("Insert MLA Level 2 Section Heading (Italic Flush Left) ({})", h2_sc))
            .clicked()
        {
            event = Some(ToolbarEvent::AddHeading(2));
        }

        let h3_sc = keybinds
            .get_shortcut(Action::InsertHeading3)
            .display_string();
        if ui
            .button(RichText::new(format!("{} H3", icons::HEADING)).color(text_col))
            .on_hover_text(format!("Insert MLA Level 3 Section Heading (Bold Centered) ({})", h3_sc))
            .clicked()
        {
            event = Some(ToolbarEvent::AddHeading(3));
        }

        ui.separator();

        // Works Cited Manager button with count
        let wc_sc = keybinds
            .get_shortcut(Action::ManageWorksCited)
            .display_string();
        let wc_label = format!(
            "{} Works Cited ({})",
            icons::BOOK_CITATIONS,
            doc.works_cited.len()
        );
        if ui
            .button(RichText::new(wc_label).color(text_col))
            .on_hover_text(format!("Manage MLA Works Cited entries ({})", wc_sc))
            .clicked()
        {
            event = Some(ToolbarEvent::OpenWorksCited);
        }

        // Add Footnote / Definition button
        let fn_label = format!("{} Note / Def ({})", icons::INFO, doc.notes.len());
        if ui
            .button(RichText::new(fn_label).color(text_col))
            .on_hover_text("Insert Explanatory Note / Definition (^N + Space)")
            .clicked()
        {
            event = Some(ToolbarEvent::AddFootnote);
        }

        // Draggable empty space between tools and window controls
        let avail_rect = ui.available_rect_before_wrap();
        let mut drag_rect = avail_rect;
        if theme.show_window_controls && drag_rect.width() > 100.0 {
            drag_rect.max.x -= 95.0;
        }
        if drag_rect.is_positive() {
            let drag_resp = ui.interact(
                drag_rect,
                ui.id().with("toolbar_drag_space"),
                egui::Sense::click_and_drag(),
            );
            if drag_resp.drag_started() {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
            }
            if drag_resp.double_clicked() {
                let is_max = ui.input(|i| i.viewport().maximized.unwrap_or(false));
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Maximized(!is_max));
            }
        }

        // Window Controls (Minimize, Maximize/Restore, Close) on the top right
        if theme.show_window_controls {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 4.0;

                // Close button
                let close_btn = ui.add(
                    egui::Button::new(
                        RichText::new("✕")
                            .size(12.0)
                            .color(text_col)
                            .strong(),
                    )
                    .min_size(egui::vec2(26.0, 22.0))
                ).on_hover_text("Close");
                if close_btn.clicked() {
                    event = Some(ToolbarEvent::CloseApp);
                }

                // Maximize / Restore button
                let is_max = ui.input(|i| i.viewport().maximized.unwrap_or(false));
                let max_char = if is_max { "🗗" } else { "🗖" };
                let max_btn = ui.add(
                    egui::Button::new(
                        RichText::new(max_char)
                            .size(11.0)
                            .color(text_col),
                    )
                    .min_size(egui::vec2(26.0, 22.0))
                ).on_hover_text(if is_max { "Restore" } else { "Maximize" });
                if max_btn.clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Maximized(!is_max));
                }

                // Minimize button
                let min_btn = ui.add(
                    egui::Button::new(
                        RichText::new("🗕")
                            .size(11.0)
                            .color(text_col),
                    )
                    .min_size(egui::vec2(26.0, 22.0))
                ).on_hover_text("Minimize");
                if min_btn.clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                }
            });
        }
    });

    event
}
