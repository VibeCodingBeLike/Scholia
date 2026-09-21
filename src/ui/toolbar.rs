use crate::fonts::icons;
use crate::keybinds::{Action, KeybindConfig};
use crate::mla_rules::MlaLinter;
use crate::model::MlaDocument;
use crate::theme::ThemeConfig;
use egui::{Color32, RichText, Ui};

pub enum ToolbarEvent {
    NewDoc,
    OpenDoc,
    SaveDoc,
    ExportDocx,
    ExportHtml,
    ExportText,
    AddParagraph,
    AddBlockquote,
    AddHeading(u8),
    InsertCitation,
    FormatTitleCase,
    OpenWorksCited,
    OpenCompliance,
    OpenSettings,
    ToggleFocusMode,
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
    let muted_col = theme.muted_text_color();
    let accent_col = theme.accent_color();

    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(6.0, 6.0);

        // Brand / Title
        ui.label(
            RichText::new("Scholia")
                .strong()
                .size(15.0)
                .color(accent_col),
        );
        ui.label(
            RichText::new("9th Ed.")
                .size(10.0)
                .italics()
                .color(theme.muted_text_color()),
        );

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
            .on_hover_text(format!("Open .mladoc file ({})", open_sc))
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

        // Export dropdown / buttons
        egui::ComboBox::from_id_salt("export_cb")
            .selected_text(
                RichText::new(format!("{} Export", icons::WORD_DOCX))
                    .strong()
                    .color(accent_col),
            )
            .show_ui(ui, |ui| {
                let docx_sc = keybinds.get_shortcut(Action::ExportDocx).display_string();
                if ui
                    .button(format!(
                        "{} Word Document (.docx)  [{}]",
                        icons::WORD_DOCX,
                        docx_sc
                    ))
                    .clicked()
                {
                    event = Some(ToolbarEvent::ExportDocx);
                }
                let html_sc = keybinds
                    .get_shortcut(Action::ExportHtmlPdf)
                    .display_string();
                if ui
                    .button(format!(
                        "{} Printable HTML / PDF  [{}]",
                        icons::HTML_PDF,
                        html_sc
                    ))
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

        // --- MLA Block Inserters ---
        let p_sc = keybinds.get_shortcut(Action::AddParagraph).display_string();
        if ui
            .button(RichText::new(format!("{} Paragraph", icons::PARAGRAPH)).color(text_col))
            .on_hover_text(format!(
                "Add double-spaced 0.5\" indented body paragraph ({})",
                p_sc
            ))
            .clicked()
        {
            event = Some(ToolbarEvent::AddParagraph);
        }

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
            .on_hover_text(format!("Insert MLA Level 1 Section Heading ({})", h1_sc))
            .clicked()
        {
            event = Some(ToolbarEvent::AddHeading(1));
        }

        let h2_sc = keybinds
            .get_shortcut(Action::InsertHeading2)
            .display_string();
        if ui
            .button(RichText::new(format!("{} H2", icons::HEADING)).color(text_col))
            .on_hover_text(format!("Insert MLA Level 2 Section Heading ({})", h2_sc))
            .clicked()
        {
            event = Some(ToolbarEvent::AddHeading(2));
        }

        let cite_sc = keybinds
            .get_shortcut(Action::InsertCitation)
            .display_string();
        if ui
            .button(RichText::new(format!("{} Citation", icons::QUOTE)).color(accent_col))
            .on_hover_text(format!(
                "Insert in-text citation e.g. (Author 42) ({})",
                cite_sc
            ))
            .clicked()
        {
            event = Some(ToolbarEvent::InsertCitation);
        }

        ui.separator();

        // Title case helper
        let tc_sc = keybinds
            .get_shortcut(Action::ConvertToMlaTitleCase)
            .display_string();
        if ui
            .button(RichText::new(format!("{} Title Case", icons::TITLE_CASE)).color(text_col))
            .on_hover_text(format!(
                "Format document title to MLA Capitalization rules ({})",
                tc_sc
            ))
            .clicked()
        {
            event = Some(ToolbarEvent::FormatTitleCase);
        }

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

        ui.separator();

        // MLA Compliance score pill
        let report = MlaLinter::inspect(doc);
        let (score_col, score_icon) = if report.score_percentage >= 95 {
            (Color32::from_rgb(50, 200, 100), icons::CHECK)
        } else if report.score_percentage >= 75 {
            (Color32::from_rgb(240, 170, 40), icons::WARNING)
        } else {
            (Color32::from_rgb(240, 70, 70), icons::TIMES)
        };

        let chk_sc = keybinds
            .get_shortcut(Action::ToggleComplianceCheck)
            .display_string();
        let comp_label = format!("{} MLA: {}%", score_icon, report.score_percentage);
        if ui
            .button(RichText::new(comp_label).strong().color(score_col))
            .on_hover_text(format!("Run MLA 9 Compliance Inspector ({})", chk_sc))
            .clicked()
        {
            event = Some(ToolbarEvent::OpenCompliance);
        }

        // Live Document Stats: Page count (estimated PDF) & Word count
        let word_count = doc.total_word_count();
        let page_count = doc.estimated_page_count();
        let stats_text = format!(
            "📄 {} {}  •  📝 {} {}",
            page_count,
            if page_count == 1 { "Page" } else { "Pages" },
            word_count,
            if word_count == 1 { "Word" } else { "Words" }
        );

        let avail = ui.available_width();
        let stats_approx_w = 210.0;
        if avail > stats_approx_w + 12.0 {
            ui.add_space(avail - stats_approx_w);
        } else {
            ui.add_space(10.0);
        }

        ui.label(
            RichText::new(stats_text)
                .size(12.5)
                .color(muted_col),
        );
    });

    event
}
