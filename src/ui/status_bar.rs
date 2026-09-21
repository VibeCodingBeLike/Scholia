use crate::fonts::icons;
use crate::mla_rules::MlaLinter;
use crate::model::MlaDocument;
use crate::theme::ThemeConfig;
use egui::{Color32, RichText, Ui};

pub enum StatusBarEvent {
    OpenCompliance,
}

pub fn render_status_bar(
    ui: &mut Ui,
    doc: &MlaDocument,
    theme: &ThemeConfig,
) -> Option<StatusBarEvent> {
    let mut event = None;
    let word_count = doc.total_word_count();
    let char_count = doc.total_char_count();
    let est_pages = doc.estimated_page_count();
    let reading_time = ((word_count as f32) / 200.0).ceil() as usize;

    let text_col = theme.muted_text_color();
    let accent_col = theme.accent_color();

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 12.0;

        // Save state indicator
        if doc.is_dirty {
            ui.colored_label(
                Color32::from_rgb(240, 160, 50),
                format!("{} Unsaved", icons::WARNING),
            );
        } else {
            ui.colored_label(
                Color32::from_rgb(80, 200, 120),
                format!("{} Saved", icons::CHECK),
            );
        }

        if let Some(path) = &doc.file_path {
            let filename = std::path::Path::new(path)
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or("Document");
            ui.label(
                RichText::new(format!("{} {}", icons::FOLDER_OPEN, filename)).color(text_col),
            );
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // MLA Compliance score pill on the footer
            let report = MlaLinter::inspect(doc);
            let (score_col, score_icon) = if report.score_percentage >= 95 {
                (Color32::from_rgb(50, 200, 100), icons::CHECK)
            } else if report.score_percentage >= 75 {
                (Color32::from_rgb(240, 170, 40), icons::WARNING)
            } else {
                (Color32::from_rgb(240, 70, 70), icons::TIMES)
            };

            let comp_label = format!("{} MLA: {}%", score_icon, report.score_percentage);
            if ui
                .button(RichText::new(comp_label).strong().size(11.5).color(score_col))
                .on_hover_text("Open MLA 9 Compliance Inspector (Ctrl+Shift+C)")
                .clicked()
            {
                event = Some(StatusBarEvent::OpenCompliance);
            }

            ui.separator();

            // Stats
            ui.label(
                RichText::new(format!("⏳ ~{} min read", reading_time.max(1)))
                    .color(text_col)
                    .size(11.5),
            );

            ui.separator();

            ui.label(
                RichText::new(format!(
                    "{} ~{} Page{} (PDF)",
                    icons::FILE_NEW,
                    est_pages,
                    if est_pages == 1 { "" } else { "s" }
                ))
                .color(accent_col)
                .strong()
                .size(11.5),
            );

            ui.separator();

            ui.label(
                RichText::new(format!("{} {} chars", icons::TEXT, char_count))
                    .color(text_col)
                    .size(11.5),
            );

            ui.separator();

            ui.label(
                RichText::new(format!("{} {} words", icons::EDIT, word_count))
                    .color(theme.text_color())
                    .strong()
                    .size(12.0),
            );
        });
    });

    event
}
