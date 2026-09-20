use crate::model::MlaDocument;
use crate::theme::ThemeConfig;
use egui::{Color32, Ui};

pub fn render_status_bar(ui: &mut Ui, doc: &MlaDocument, theme: &ThemeConfig) {
    let word_count = doc.total_word_count();
    let char_count = doc.total_char_count();
    let est_pages = doc.estimated_page_count();
    let reading_time = ((word_count as f32) / 200.0).ceil() as usize;

    let text_col = theme.muted_text_color();
    let accent_col = theme.accent_color();

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 16.0;

        // Save state indicator
        if doc.is_dirty {
            ui.colored_label(Color32::from_rgb(240, 160, 50), "● Unsaved Changes");
        } else {
            ui.colored_label(Color32::from_rgb(80, 200, 120), "✓ Saved");
        }

        if let Some(path) = &doc.file_path {
            let filename = std::path::Path::new(path)
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or("Document");
            ui.label(egui::RichText::new(format!("📁 {}", filename)).color(text_col));
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Stats
            ui.label(
                egui::RichText::new(format!("⏳ ~{} min read", reading_time.max(1)))
                    .color(text_col)
                    .size(11.5),
            );

            ui.separator();

            ui.label(
                egui::RichText::new(format!(
                    "📄 ~{} Page{}",
                    est_pages,
                    if est_pages == 1 { "" } else { "s" }
                ))
                .color(accent_col)
                .strong()
                .size(11.5),
            );

            ui.separator();

            ui.label(
                egui::RichText::new(format!("🔤 {} chars", char_count))
                    .color(text_col)
                    .size(11.5),
            );

            ui.separator();

            ui.label(
                egui::RichText::new(format!("📝 {} words", word_count))
                    .color(theme.text_color())
                    .strong()
                    .size(12.0),
            );
        });
    });
}
