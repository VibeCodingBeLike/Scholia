use crate::model::works_cited::{SourceType, WorksCitedEntry};
use crate::theme::ThemeConfig;
use egui::{RichText, Window};

pub struct WorksCitedModalState {
    pub is_open: bool,
    pub editing_entry: WorksCitedEntry,
    pub editing_index: Option<usize>,
}

impl Default for WorksCitedModalState {
    fn default() -> Self {
        Self {
            is_open: false,
            editing_entry: WorksCitedEntry::new_empty(),
            editing_index: None,
        }
    }
}

impl WorksCitedModalState {
    pub fn open_new(&mut self) {
        self.is_open = true;
        self.editing_entry = WorksCitedEntry::new_empty();
        self.editing_index = None;
    }

    pub fn open_edit(&mut self, index: usize, entry: &WorksCitedEntry) {
        self.is_open = true;
        self.editing_entry = entry.clone();
        self.editing_index = Some(index);
    }
}

pub fn render_works_cited_modal(
    ctx: &egui::Context,
    state: &mut WorksCitedModalState,
    works_cited: &mut Vec<WorksCitedEntry>,
    theme: &ThemeConfig,
) {
    if !state.is_open {
        return;
    }

    let mut open = true;
    let mut close_modal = false;
    let text_col = theme.text_color();
    let accent_col = theme.accent_color();

    Window::new(if state.editing_index.is_some() {
        "Edit Works Cited Entry (MLA 9th Edition)"
    } else {
        "Add New Works Cited Entry (MLA 9th Edition)"
    })
    .open(&mut open)
    .resizable(true)
    .frame(theme.modal_frame())
    .default_width(580.0)
    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
    .show(ctx, |ui| {
        ui.spacing_mut().item_spacing.y = 8.0;

        // Source Type Selector
        ui.horizontal(|ui| {
            ui.label(RichText::new("Source Type:").strong().color(text_col));
            egui::ComboBox::from_id_salt("source_type_cb")
                .selected_text(state.editing_entry.source_type.display_name())
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut state.editing_entry.source_type,
                        SourceType::BookOrStandalone,
                        SourceType::BookOrStandalone.display_name(),
                    );
                    ui.selectable_value(
                        &mut state.editing_entry.source_type,
                        SourceType::ArticleOrChapter,
                        SourceType::ArticleOrChapter.display_name(),
                    );
                    ui.selectable_value(
                        &mut state.editing_entry.source_type,
                        SourceType::JournalArticle,
                        SourceType::JournalArticle.display_name(),
                    );
                    ui.selectable_value(
                        &mut state.editing_entry.source_type,
                        SourceType::WebPage,
                        SourceType::WebPage.display_name(),
                    );
                    ui.selectable_value(
                        &mut state.editing_entry.source_type,
                        SourceType::Custom,
                        SourceType::Custom.display_name(),
                    );
                });
        });

        ui.separator();

        if state.editing_entry.source_type == SourceType::Custom {
            ui.label(RichText::new("Paste Pre-Formatted MLA Citation:").color(text_col));
            ui.add(
                egui::TextEdit::multiline(&mut state.editing_entry.raw_text)
                    .desired_rows(4)
                    .desired_width(f32::INFINITY),
            );
            ui.label(
                RichText::new(
                    "Tip: Use asterisks around book/journal titles for italics: *The Great Gatsby*",
                )
                .size(11.0)
                .color(theme.muted_text_color()),
            );
        } else {
            // MLA 9 Nine Core Elements Form
            egui::Grid::new("mla9_core_elements_grid")
                .num_columns(2)
                .spacing([12.0, 8.0])
                .striped(true)
                .show(ui, |ui| {
                    // 1. Author
                    ui.label(RichText::new("1. Author(s):").strong().color(text_col));
                    ui.add(
                        egui::TextEdit::singleline(&mut state.editing_entry.author)
                            .hint_text("e.g. Morrison, Toni or Smith, John, and Jane Doe")
                            .desired_width(400.0),
                    );
                    ui.end_row();

                    // 2. Title of Source
                    ui.label(
                        RichText::new("2. Title of Source:")
                            .strong()
                            .color(text_col),
                    );
                    ui.add(
                        egui::TextEdit::singleline(&mut state.editing_entry.title_of_source)
                            .hint_text("e.g. Beloved or Reading the Modern Metropolis")
                            .desired_width(400.0),
                    );
                    ui.end_row();

                    // 3. Container Title
                    ui.label(
                        RichText::new("3. Container Title:")
                            .strong()
                            .color(text_col),
                    );
                    ui.add(
                        egui::TextEdit::singleline(&mut state.editing_entry.container_title)
                            .hint_text("e.g. The Atlantic or Critical Inquiry or JSTOR")
                            .desired_width(400.0),
                    );
                    ui.end_row();

                    // 4. Other Contributors
                    ui.label(RichText::new("4. Contributors:").color(theme.muted_text_color()));
                    ui.add(
                        egui::TextEdit::singleline(&mut state.editing_entry.other_contributors)
                            .hint_text("e.g. edited by John Doe or translated by Jane Doe")
                            .desired_width(400.0),
                    );
                    ui.end_row();

                    // 5. Version
                    ui.label(
                        RichText::new("5. Version / Edition:").color(theme.muted_text_color()),
                    );
                    ui.add(
                        egui::TextEdit::singleline(&mut state.editing_entry.version)
                            .hint_text("e.g. 9th ed. or Expanded version")
                            .desired_width(400.0),
                    );
                    ui.end_row();

                    // 6. Number
                    ui.label(RichText::new("6. Number / Volume:").color(theme.muted_text_color()));
                    ui.add(
                        egui::TextEdit::singleline(&mut state.editing_entry.number)
                            .hint_text("e.g. vol. 12, no. 4")
                            .desired_width(400.0),
                    );
                    ui.end_row();

                    // 7. Publisher
                    ui.label(RichText::new("7. Publisher:").color(theme.muted_text_color()));
                    ui.add(
                        egui::TextEdit::singleline(&mut state.editing_entry.publisher)
                            .hint_text("e.g. Oxford UP or Penguin Books")
                            .desired_width(400.0),
                    );
                    ui.end_row();

                    // 8. Publication Date
                    ui.label(
                        RichText::new("8. Publication Date:")
                            .strong()
                            .color(text_col),
                    );
                    ui.add(
                        egui::TextEdit::singleline(&mut state.editing_entry.pub_date)
                            .hint_text("e.g. 2021 or 15 May 2023")
                            .desired_width(400.0),
                    );
                    ui.end_row();

                    // 9. Location
                    ui.label(RichText::new("9. Location:").color(text_col));
                    ui.add(
                        egui::TextEdit::singleline(&mut state.editing_entry.location)
                            .hint_text("e.g. pp. 45-67 or https://... or doi:10.10...")
                            .desired_width(400.0),
                    );
                    ui.end_row();
                });
        }

        ui.separator();

        // Live MLA 9 Formatted Preview
        ui.group(|ui| {
            ui.label(
                RichText::new("MLA 9 Formatted Preview (with Hanging Indent):")
                    .strong()
                    .color(accent_col),
            );
            let preview = state.editing_entry.format_markdown();
            if preview.trim().is_empty() {
                ui.label(
                    RichText::new("(Fill in the fields above to preview formatted entry)")
                        .italics()
                        .color(theme.muted_text_color()),
                );
            } else {
                ui.horizontal(|ui| {
                    ui.add_space(20.0);
                    ui.label(RichText::new(preview).color(text_col));
                });
                let in_text_example = state.editing_entry.in_text_citation_prompt("42");
                ui.label(
                    RichText::new(format!("In-Text Citation Example: {}", in_text_example))
                        .size(11.5)
                        .color(theme.muted_text_color()),
                );
            }
        });

        ui.separator();

        // Bottom action buttons
        ui.horizontal(|ui| {
            if ui.button(RichText::new("💾 Save Entry").strong().color(accent_col)).clicked() {
                if let Some(idx) = state.editing_index {
                    if idx < works_cited.len() {
                        works_cited[idx] = state.editing_entry.clone();
                    }
                } else {
                    works_cited.push(state.editing_entry.clone());
                }
                // Auto sort
                works_cited.sort_by_key(|a| a.sort_key());
                close_modal = true;
            }

            if let Some(idx) = state.editing_index {
                if ui
                    .button(RichText::new("🗑 Delete Entry").color(egui::Color32::from_rgb(220, 80, 80)))
                    .clicked()
                {
                    if idx < works_cited.len() {
                        works_cited.remove(idx);
                    }
                    close_modal = true;
                }
            }

            if ui.button("Cancel").clicked() {
                close_modal = true;
            }
        });
    });

    if !open || close_modal {
        state.is_open = false;
    }
}
