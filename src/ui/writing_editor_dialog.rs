use crate::fonts::icons;
use crate::model::MlaDocument;
use crate::theme::ThemeConfig;
use crate::writing_editor::{
    EditorSettings, EditorTarget, ReplacementAction, SuggestionCategory, WritingEditor,
};
use egui::{Color32, RichText, ScrollArea, Window};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EditorFilterTab {
    #[default]
    All,
    Spelling,
    Grammar,
    Punctuation,
    Style,
}

#[derive(Default)]
pub struct WritingEditorModalState {
    pub is_open: bool,
    pub selected_filter: EditorFilterTab,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WritingEditorModalEvent {
    OpenEditorSettings,
    DictionaryModified,
}

pub fn render_writing_editor_modal(
    ctx: &egui::Context,
    state: &mut WritingEditorModalState,
    doc: &mut MlaDocument,
    settings: &mut EditorSettings,
    theme: &ThemeConfig,
) -> Option<WritingEditorModalEvent> {
    if !state.is_open {
        return None;
    }

    let mut open = true;
    let mut close_modal = false;
    let mut modal_event = None;

    let report = WritingEditor::inspect(doc, settings);

    Window::new("Writing Editor — Spell, Grammar & Academic Style")
        .open(&mut open)
        .resizable(true)
        .frame(theme.modal_frame())
        .default_width(660.0)
        .default_height(560.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            // 1. Header: Editor Score + Language Badge + Quick Settings Button
            ui.horizontal(|ui| {
                let score_color = if report.editor_score >= 95 {
                    Color32::from_rgb(40, 190, 90)
                } else if report.editor_score >= 75 {
                    Color32::from_rgb(230, 160, 40)
                } else {
                    Color32::from_rgb(230, 60, 60)
                };

                ui.label(
                    RichText::new(format!("Editor Score: {}%", report.editor_score))
                        .size(18.0)
                        .strong()
                        .color(score_color),
                );

                ui.separator();

                ui.label(
                    RichText::new(format!(
                        "{} suggestion{} remaining",
                        report.total_suggestions(),
                        if report.total_suggestions() == 1 {
                            ""
                        } else {
                            "s"
                        }
                    ))
                    .color(theme.muted_text_color()),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .button("⚙ Editor Settings")
                        .on_hover_text("Configure language (US / CA / UK), rules, and custom dictionary")
                        .clicked()
                    {
                        modal_event = Some(WritingEditorModalEvent::OpenEditorSettings);
                    }

                    ui.label(
                        RichText::new(format!("🌐 {}", settings.language.short_code()))
                            .size(11.5)
                            .strong()
                            .color(theme.accent_color()),
                    );
                });
            });

            ui.add_space(4.0);

            let progress = (report.editor_score as f32) / 100.0;
            ui.add(egui::ProgressBar::new(progress).show_percentage());

            ui.add_space(6.0);

            // 2. Document Readability & Writing Insights Strip
            ui.group(|ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing.x = 14.0;
                    ui.label(
                        RichText::new("📊 Writing Insights:")
                            .size(11.5)
                            .strong()
                            .color(theme.accent_color()),
                    );
                    ui.label(
                        RichText::new(format!(
                            "Reading Ease: {:.0}/100",
                            report.flesch_reading_ease
                        ))
                        .size(11.5)
                        .color(theme.text_color()),
                    );
                    ui.separator();
                    ui.label(
                        RichText::new(format!(
                            "Grade Level: {:.1}",
                            report.flesch_kincaid_grade
                        ))
                        .size(11.5)
                        .color(theme.text_color()),
                    );
                    ui.separator();
                    ui.label(
                        RichText::new(format!(
                            "Avg Sentence: {:.1} words",
                            report.avg_sentence_length
                        ))
                        .size(11.5)
                        .color(theme.text_color()),
                    );
                    ui.separator();
                    ui.label(
                        RichText::new(format!(
                            "Vocabulary Diversity: {:.0}%",
                            report.vocabulary_diversity_pct
                        ))
                        .size(11.5)
                        .color(theme.text_color()),
                    );
                });
            });

            ui.add_space(4.0);

            // 3. Category Filter Bar
            ui.horizontal_wrapped(|ui| {
                ui.selectable_value(
                    &mut state.selected_filter,
                    EditorFilterTab::All,
                    format!("All ({})", report.total_suggestions()),
                );
                ui.selectable_value(
                    &mut state.selected_filter,
                    EditorFilterTab::Spelling,
                    format!("🔴 Spelling ({})", report.spelling_count),
                );
                ui.selectable_value(
                    &mut state.selected_filter,
                    EditorFilterTab::Grammar,
                    format!("🔵 Grammar ({})", report.grammar_count),
                );
                ui.selectable_value(
                    &mut state.selected_filter,
                    EditorFilterTab::Punctuation,
                    format!("🟡 Punctuation ({})", report.punctuation_count),
                );
                ui.selectable_value(
                    &mut state.selected_filter,
                    EditorFilterTab::Style,
                    format!("🟣 Clarity & Style ({})", report.style_count),
                );
            });

            ui.separator();

            // Filter issues according to active tab
            let filtered_issues: Vec<_> = report
                .issues
                .iter()
                .filter(|issue| match state.selected_filter {
                    EditorFilterTab::All => true,
                    EditorFilterTab::Spelling => issue.category == SuggestionCategory::Spelling,
                    EditorFilterTab::Grammar => issue.category == SuggestionCategory::Grammar,
                    EditorFilterTab::Punctuation => {
                        issue.category == SuggestionCategory::Punctuation
                    }
                    EditorFilterTab::Style => issue.category == SuggestionCategory::Style,
                })
                .cloned()
                .collect();

            // Defer mutations collected during card rendering
            let mut accepted_action: Option<(crate::writing_editor::EditorIssue, ReplacementAction)> =
                None;
            let mut ignored_issue_key: Option<String> = None;
            let mut ignored_word_all: Option<String> = None;
            let mut add_dict_word: Option<String> = None;
            let mut focus_block_idx: Option<usize> = None;

            // 4. Scrollable Suggestions List
            ScrollArea::vertical()
                .max_height(330.0)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if filtered_issues.is_empty() {
                        ui.vertical_centered(|ui| {
                            ui.add_space(36.0);
                            ui.label(
                                RichText::new(format!(
                                    "{} Looking good! No suggestions in this category.",
                                    icons::CHECK
                                ))
                                .size(15.0)
                                .color(Color32::from_rgb(60, 200, 100))
                                .strong(),
                            );
                            ui.add_space(6.0);
                            ui.label(
                                RichText::new(
                                    "Keep writing distraction-free — the Editor checks your spelling, grammar, and academic style whenever you're ready.",
                                )
                                .size(12.0)
                                .color(theme.muted_text_color()),
                            );
                            ui.add_space(36.0);
                        });
                    } else {
                        for issue in &filtered_issues {
                            ui.group(|ui| {
                                let (badge_col, badge_icon) = match issue.category {
                                    SuggestionCategory::Spelling => {
                                        (Color32::from_rgb(235, 85, 85), "🔴")
                                    }
                                    SuggestionCategory::Grammar => {
                                        (Color32::from_rgb(75, 165, 245), "🔵")
                                    }
                                    SuggestionCategory::Punctuation => {
                                        (Color32::from_rgb(240, 180, 50), "🟡")
                                    }
                                    SuggestionCategory::Style => {
                                        (Color32::from_rgb(185, 130, 245), "🟣")
                                    }
                                };

                                // Top row: Category + Location + Jump button
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new(format!(
                                            "{} {}",
                                            badge_icon,
                                            issue.category.display_name()
                                        ))
                                        .strong()
                                        .size(12.0)
                                        .color(badge_col),
                                    );

                                    ui.label(
                                        RichText::new(format!("• {}", issue.location_label))
                                            .size(11.5)
                                            .strong()
                                            .color(theme.muted_text_color()),
                                    );

                                    ui.label(
                                        RichText::new(format!("“{}”", issue.flagged_text))
                                            .size(12.5)
                                            .strong()
                                            .color(theme.text_color()),
                                    );

                                    if let EditorTarget::Block(b_idx) = issue.target {
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                if ui
                                                    .small_button("🔍 Jump to Block")
                                                    .on_hover_text("Focus this block in the editor")
                                                    .clicked()
                                                {
                                                    focus_block_idx = Some(b_idx);
                                                }
                                            },
                                        );
                                    }
                                });

                                // Context snippet
                                ui.horizontal_wrapped(|ui| {
                                    ui.spacing_mut().item_spacing.x = 0.0;
                                    ui.label(
                                        RichText::new(&issue.context_before)
                                            .size(11.5)
                                            .italics()
                                            .color(theme.muted_text_color()),
                                    );
                                    ui.label(
                                        RichText::new(&issue.flagged_text)
                                            .size(11.5)
                                            .strong()
                                            .underline()
                                            .color(badge_col),
                                    );
                                    ui.label(
                                        RichText::new(&issue.context_after)
                                            .size(11.5)
                                            .italics()
                                            .color(theme.muted_text_color()),
                                    );
                                });

                                // Explanation message
                                ui.label(
                                    RichText::new(&issue.message)
                                        .size(12.0)
                                        .color(theme.text_color()),
                                );

                                ui.add_space(3.0);

                                // Action buttons: Yes (Accept suggestions) / No (Ignore) / Add to Dictionary
                                ui.horizontal_wrapped(|ui| {
                                    ui.spacing_mut().item_spacing.x = 8.0;

                                    for (s_idx, sug) in issue.suggestions.iter().enumerate() {
                                        let label = if s_idx == 0 {
                                            match sug {
                                                ReplacementAction::Replace(rep) => {
                                                    format!("✓ Yes: \"{}\"", rep)
                                                }
                                                ReplacementAction::InsertAfter(ins) => {
                                                    format!("✓ Yes: Insert \"{}\"", ins)
                                                }
                                                ReplacementAction::Remove => {
                                                    "✓ Yes: Remove".to_string()
                                                }
                                            }
                                        } else {
                                            sug.button_label()
                                        };

                                        let btn_text = if s_idx == 0 {
                                            RichText::new(label)
                                                .strong()
                                                .color(Color32::from_rgb(70, 215, 120))
                                        } else {
                                            RichText::new(label).color(theme.accent_color())
                                        };

                                        if ui
                                            .button(btn_text)
                                            .on_hover_text("Accept this suggestion and update document")
                                            .clicked()
                                        {
                                            accepted_action =
                                                Some((issue.clone(), sug.clone()));
                                        }
                                    }

                                    if ui
                                        .button(
                                            RichText::new("✕ No (Ignore)")
                                                .color(theme.muted_text_color()),
                                        )
                                        .on_hover_text("Dismiss this suggestion")
                                        .clicked()
                                    {
                                        ignored_issue_key = Some(issue.issue_key.clone());
                                    }

                                    if issue.can_add_to_dictionary {
                                        if ui
                                            .button(
                                                RichText::new("✕ Ignore All")
                                                    .size(11.5)
                                                    .color(theme.muted_text_color()),
                                            )
                                            .on_hover_text(
                                                "Ignore all occurrences of this word for this session",
                                            )
                                            .clicked()
                                        {
                                            ignored_word_all = Some(issue.flagged_text.clone());
                                        }

                                        if ui
                                            .button(
                                                RichText::new(format!(
                                                    "📖 + Add \"{}\" to Dictionary",
                                                    issue.flagged_text
                                                ))
                                                .strong()
                                                .color(theme.accent_color()),
                                            )
                                            .on_hover_text(
                                                "Save this word to your custom dictionary so it is never flagged again",
                                            )
                                            .clicked()
                                        {
                                            add_dict_word = Some(issue.flagged_text.clone());
                                        }
                                    }
                                });
                            });
                            ui.add_space(4.0);
                        }
                    }
                });

            // Apply deferred user actions
            if let Some((issue, action)) = accepted_action {
                WritingEditor::apply_suggestion(doc, &issue, &action);
            }
            if let Some(key) = ignored_issue_key {
                settings.ignore_issue(&key);
            }
            if let Some(word) = ignored_word_all {
                settings.ignore_word_for_session(&word);
            }
            if let Some(word) = add_dict_word {
                if settings.add_to_dictionary(&word) {
                    modal_event = Some(WritingEditorModalEvent::DictionaryModified);
                }
            }
            if let Some(b_idx) = focus_block_idx {
                if b_idx < doc.blocks.len() {
                    doc.active_block_idx = b_idx;
                    doc.requested_focus_block_idx = Some(b_idx);
                }
            }

            ui.separator();

            // 5. Bottom Controls
            ui.horizontal(|ui| {
                if ui.button("Close").clicked() {
                    close_modal = true;
                }

                let ignored_total =
                    settings.ignored_issues.len() + settings.ignored_words_session.len();
                if ignored_total > 0
                    && ui
                        .button(format!("↺ Restore Ignored ({})", ignored_total))
                        .on_hover_text("Show suggestions you previously dismissed with 'No (Ignore)'")
                        .clicked()
                {
                    settings.clear_ignored();
                }

                let has_actionable = report.issues.iter().any(|i| !i.suggestions.is_empty());
                if has_actionable {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .button(
                                RichText::new("⚡ Accept All Top Suggestions")
                                    .strong()
                                    .color(theme.accent_color()),
                            )
                            .on_hover_text(
                                "Automatically apply the primary suggestion for every remaining issue",
                            )
                            .clicked()
                        {
                            WritingEditor::apply_all_top_suggestions(doc, settings);
                        }
                    });
                }
            });
        });

    if !open || close_modal {
        state.is_open = false;
    }

    modal_event
}
