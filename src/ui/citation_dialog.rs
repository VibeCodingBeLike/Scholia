use crate::model::works_cited::WorksCitedEntry;
use crate::theme::ThemeConfig;
use egui::{RichText, Window};

#[derive(Default)]
pub struct CitationModalState {
    pub is_open: bool,
    pub selected_source_index: usize,
    pub page_number: String,
    pub custom_author: String,
    pub target_block_index: Option<usize>,
}

impl CitationModalState {
    pub fn open(&mut self, block_index: Option<usize>) {
        self.is_open = true;
        self.page_number.clear();
        self.target_block_index = block_index;
    }
}

pub fn render_citation_modal(
    ctx: &egui::Context,
    state: &mut CitationModalState,
    works_cited: &[WorksCitedEntry],
    theme: &ThemeConfig,
    on_insert: &mut Option<String>,
) {
    if !state.is_open {
        return;
    }

    let mut open = true;
    let mut close_modal = false;
    let text_col = theme.text_color();
    let accent_col = theme.accent_color();

    let screen_w = ctx.input(|i| i.viewport().inner_rect.map(|r| r.width()).unwrap_or(1000.0));
    let screen_h = ctx.input(|i| i.viewport().inner_rect.map(|r| r.height()).unwrap_or(750.0));
    let max_w = (screen_w * 0.85).clamp(400.0, 560.0);
    let max_h = (screen_h * 0.85).round();

    Window::new("Insert MLA In-Text Citation")
        .open(&mut open)
        .resizable(false)
        .frame(theme.modal_frame())
        .default_width(450.0)
        .max_width(max_w)
        .max_height(max_h)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing.y = 10.0;

            ui.label(
                RichText::new(
                    "MLA 9 in-text citations follow the Author-Page format: e.g. (Smith 42)",
                )
                .size(11.5)
                .color(theme.muted_text_color()),
            );

            if !works_cited.is_empty() {
                ui.label(
                    RichText::new("Select source from Works Cited:")
                        .strong()
                        .color(text_col),
                );

                let selected_text = if state.selected_source_index < works_cited.len() {
                    let entry = &works_cited[state.selected_source_index];
                    let auth = if !entry.author.is_empty() {
                        &entry.author
                    } else {
                        &entry.title_of_source
                    };
                    format!("{}: {}", auth, entry.title_of_source)
                } else {
                    "Custom Author / Title".to_string()
                };

                egui::ComboBox::from_id_salt("citation_source_cb")
                    .selected_text(selected_text)
                    .width(400.0)
                    .show_ui(ui, |ui| {
                        for (i, entry) in works_cited.iter().enumerate() {
                            let label = if !entry.author.is_empty() {
                                format!("{}: {}", entry.author, entry.title_of_source)
                            } else {
                                entry.title_of_source.clone()
                            };
                            ui.selectable_value(&mut state.selected_source_index, i, label);
                        }
                    });
            } else {
                ui.label(
                    RichText::new("No Works Cited entries found yet.")
                        .color(egui::Color32::from_rgb(230, 150, 40)),
                );
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Author / Short Title:")
                            .strong()
                            .color(text_col),
                    );
                    ui.add(
                        egui::TextEdit::singleline(&mut state.custom_author)
                            .hint_text("e.g. Smith or \"Modern Art\""),
                    );
                });
            }

            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Page Number or Range:")
                        .strong()
                        .color(text_col),
                );
                ui.add(
                    egui::TextEdit::singleline(&mut state.page_number)
                        .hint_text("e.g. 42 or 12-15"),
                );
            });

            // Preview
            let formatted_citation = if !works_cited.is_empty()
                && state.selected_source_index < works_cited.len()
            {
                works_cited[state.selected_source_index].in_text_citation_prompt(&state.page_number)
            } else {
                let author = if state.custom_author.trim().is_empty() {
                    "Author"
                } else {
                    state.custom_author.trim()
                };
                let page_part = if state.page_number.trim().is_empty() {
                    String::new()
                } else {
                    format!(" {}", state.page_number.trim())
                };
                format!("({}{})", author, page_part)
            };

            ui.group(|ui| {
                ui.label(
                    RichText::new("Citation to Insert:")
                        .size(11.5)
                        .color(theme.muted_text_color()),
                );
                ui.add(
                    egui::Label::new(
                        RichText::new(&formatted_citation)
                            .strong()
                            .size(14.0)
                            .color(accent_col),
                    )
                    .wrap(),
                );
            });

            ui.separator();

            ui.horizontal(|ui| {
                if ui
                    .button(RichText::new("📌 Insert Into Document").strong())
                    .clicked()
                {
                    *on_insert = Some(formatted_citation.clone());
                    close_modal = true;
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
