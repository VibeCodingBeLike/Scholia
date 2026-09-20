use crate::fonts::{doc_font, icons};
use crate::model::format_current_mla_date;
use crate::model::MlaDocument;
use crate::theme::ThemeConfig;
use egui::{RichText, Ui};

pub enum EditorAction {
    OpenCitationModal(Option<usize>),
    OpenWorksCitedModal(Option<usize>),
}

pub fn render_editor_page(
    ui: &mut Ui,
    doc: &mut MlaDocument,
    theme: &ThemeConfig,
) -> Option<EditorAction> {
    let mut action = None;
    let text_col = theme.text_color();
    let muted_col = theme.muted_text_color();
    let accent_col = theme.accent_color();

    // Ensure body text is populated
    doc.ensure_body_synced();

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add_space(20.0);

            // Compute centered page dimensions (Standard 8.5" x 11" Paper at 96 DPI: 816px wide)
            let total_avail = ui.available_width();
            let page_width = 816.0f32.min(total_avail - 48.0).max(420.0);
            let margin_left = ((total_avail - page_width) / 2.0).max(0.0);

            // Center container
            ui.horizontal(|ui| {
                if margin_left > 0.0 {
                    ui.add_space(margin_left);
                }

                // Vertical column holding the pages
                ui.vertical(|ui| {
                    ui.set_width(page_width);
                    ui.set_min_width(page_width);
                    ui.set_max_width(page_width);

                    // ==========================================
                    // 📄 PAGE 1: THE MANUSCRIPT
                    // ==========================================
                    let page_margin = 60; // ~ 1-inch visual margin
                    let printable_width = page_width - (page_margin as f32 * 2.0);

                    let page_frame = egui::Frame::new()
                        .fill(theme.page_fill_color())
                        .stroke(theme.page_stroke())
                        .corner_radius(6.0)
                        .inner_margin(egui::Margin::symmetric(page_margin, page_margin));

                    page_frame.show(ui, |ui| {
                        ui.set_width(printable_width);
                        ui.set_min_width(printable_width);
                        ui.set_max_width(printable_width);
                        ui.spacing_mut().item_spacing.y = 8.0;

                        // --- RUNNING HEAD (Top-Right: LastName 1) ---
                        let derived_last = doc.header.derived_last_name();
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                            ui.label(
                                RichText::new(format!("{} 1", derived_last))
                                    .font(doc_font(15.0))
                                    .color(muted_col),
                            );
                        });

                        ui.add_space(10.0);

                        // --- FIRST PAGE HEADING (Flush Left, 4 Double-Spaced Lines) ---
                        // Seamless borderless text lines (NO FORM LABELS)
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 6.0;

                            // Student Name
                            let s_name = ui.add(
                                egui::TextEdit::singleline(&mut doc.header.student_name)
                                    .font(doc_font(16.0))
                                    .text_color(text_col)
                                    .frame(egui::Frame::NONE)
                                    .hint_text(RichText::new("Student Full Name").italics().color(muted_col))
                                    .desired_width(printable_width),
                            );
                            if s_name.changed() {
                                doc.is_dirty = true;
                            }

                            // Instructor Name
                            let i_name = ui.add(
                                egui::TextEdit::singleline(&mut doc.header.instructor_name)
                                    .font(doc_font(16.0))
                                    .text_color(text_col)
                                    .frame(egui::Frame::NONE)
                                    .hint_text(RichText::new("Instructor Name (e.g. Professor Robert Smith)").italics().color(muted_col))
                                    .desired_width(printable_width),
                            );
                            if i_name.changed() {
                                doc.is_dirty = true;
                            }

                            // Course Title/Number
                            let c_name = ui.add(
                                egui::TextEdit::singleline(&mut doc.header.course)
                                    .font(doc_font(16.0))
                                    .text_color(text_col)
                                    .frame(egui::Frame::NONE)
                                    .hint_text(RichText::new("Course (e.g. ENG 101: Composition)").italics().color(muted_col))
                                    .desired_width(printable_width),
                            );
                            if c_name.changed() {
                                doc.is_dirty = true;
                            }

                            // Date in MLA Format (Day Month Year)
                            ui.horizontal(|ui| {
                                let d_name = ui.add(
                                    egui::TextEdit::singleline(&mut doc.header.date)
                                        .font(doc_font(16.0))
                                        .text_color(text_col)
                                        .frame(egui::Frame::NONE)
                                        .hint_text(RichText::new("Date (e.g. 20 September 2026)").italics().color(muted_col))
                                        .desired_width(260.0),
                                );
                                if d_name.changed() {
                                    doc.is_dirty = true;
                                }

                                if ui.small_button(format!("{} Today", icons::CALENDAR)).on_hover_text("Insert current date in MLA format").clicked() {
                                    doc.header.date = format_current_mla_date();
                                    doc.is_dirty = true;
                                }
                            });
                        });

                        ui.add_space(16.0);

                        // --- PAPER TITLE (Centered, Standard 12pt, Not Bold) ---
                        ui.vertical_centered(|ui| {
                            let title_edit = ui.add(
                                egui::TextEdit::singleline(&mut doc.title)
                                    .font(doc_font(16.0))
                                    .text_color(text_col)
                                    .frame(egui::Frame::NONE)
                                    .hint_text(RichText::new("Title of Your Paper (MLA Title Case)").italics().color(muted_col))
                                    .desired_width(printable_width),
                            );
                            if title_edit.changed() {
                                doc.is_dirty = true;
                            }
                        });

                        ui.add_space(20.0);

                        // --- CONTINUOUS BODY TEXT EDITOR ---
                        // Type directly onto the page! True word-processing experience.
                        let body_edit = ui.add(
                            egui::TextEdit::multiline(&mut doc.body)
                                .font(doc_font(16.0))
                                .text_color(text_col)
                                .frame(egui::Frame::NONE) // Borderless and transparent!
                                .desired_width(printable_width)
                                .desired_rows(24)
                                .lock_focus(true)
                                .hint_text(RichText::new("Begin typing your MLA paper here...\n\nPress Enter twice to start a new paragraph (0.5\" indent is applied automatically in MLA format).\nUse Ctrl+Shift+C to insert parenthetical citations, e.g. (Smith 42).\nFor long quotations (> 4 lines), prefix with '> ' for an indented blockquote.").italics().color(muted_col)),
                        );

                        if body_edit.changed() {
                            doc.is_dirty = true;
                            doc.sync_blocks_from_body();
                        }
                    });

                    // ==========================================
                    // 📚 PAGE 2: WORKS CITED PAGE
                    // ==========================================
                    ui.add_space(32.0);

                    // Page break separator line
                    ui.horizontal(|ui| {
                        ui.add_space((page_width - 240.0) / 2.0);
                        ui.label(
                            RichText::new("────── Page Break: Works Cited ──────")
                                .size(11.5)
                                .italics()
                                .color(muted_col),
                        );
                    });

                    ui.add_space(12.0);

                    let wc_frame = egui::Frame::new()
                        .fill(theme.page_fill_color())
                        .stroke(theme.page_stroke())
                        .corner_radius(6.0)
                        .inner_margin(egui::Margin::symmetric(page_margin, page_margin));

                    wc_frame.show(ui, |ui| {
                        ui.set_width(printable_width);
                        ui.set_min_width(printable_width);
                        ui.set_max_width(printable_width);
                        ui.spacing_mut().item_spacing.y = 8.0;

                        // Running head for Works Cited page
                        let derived_last = doc.header.derived_last_name();
                        let est_pages = doc.estimated_page_count();
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                            ui.label(
                                RichText::new(format!("{} {}", derived_last, est_pages.max(2)))
                                    .font(doc_font(15.0))
                                    .color(muted_col),
                            );
                        });

                        ui.add_space(6.0);

                        // Centered "Works Cited" title
                        ui.vertical_centered(|ui| {
                            let wc_title = if doc.works_cited.len() == 1 {
                                "Work Cited"
                            } else {
                                "Works Cited"
                            };
                            ui.label(
                                RichText::new(wc_title)
                                    .font(doc_font(16.0))
                                    .color(text_col),
                            );
                        });

                        ui.add_space(14.0);

                        // Works Cited Entries (Formatted with 0.5-inch Hanging Indent!)
                        if doc.works_cited.is_empty() {
                            ui.vertical_centered(|ui| {
                                ui.label(
                                    RichText::new("No entries in Works Cited yet.")
                                        .font(doc_font(14.0))
                                        .italics()
                                        .color(muted_col),
                                );
                            });
                        } else {
                            let mut entry_to_edit = None;
                            let mut entry_to_delete = None;

                            for (idx, entry) in doc.works_cited.iter().enumerate() {
                                let formatted = entry.format_markdown();

                                ui.horizontal_top(|ui| {
                                    // 0.5-inch Hanging Indent: First line flush left, subsequent lines indented
                                    ui.label(
                                        RichText::new(format!("{}.", idx + 1))
                                            .font(doc_font(14.0))
                                            .color(muted_col),
                                    );

                                    ui.add_space(4.0);

                                    let _entry_lbl = ui.add(
                                        egui::Label::new(
                                            RichText::new(formatted)
                                                .font(doc_font(15.0))
                                                .color(text_col),
                                        )
                                        .wrap(),
                                    );

                                    // Action buttons on the side
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.small_button(icons::TRASH).on_hover_text("Delete entry").clicked() {
                                            entry_to_delete = Some(idx);
                                        }
                                        if ui.small_button(format!("{} Edit", icons::EDIT)).clicked() {
                                            entry_to_edit = Some(idx);
                                        }
                                    });
                                });

                                ui.add_space(6.0);
                            }

                            if let Some(idx) = entry_to_delete {
                                doc.works_cited.remove(idx);
                                doc.is_dirty = true;
                            }
                            if let Some(idx) = entry_to_edit {
                                action = Some(EditorAction::OpenWorksCitedModal(Some(idx)));
                            }
                        }

                        // Bottom Works Cited Action Bar
                        ui.add_space(14.0);
                        ui.horizontal(|ui| {
                            if ui.button(RichText::new(format!("{} Add MLA 9 Source Entry", icons::PLUS)).strong().color(accent_col)).clicked() {
                                action = Some(EditorAction::OpenWorksCitedModal(None));
                            }

                            if doc.works_cited.len() > 1 && ui.button(format!("{} Sort Alphabetically", icons::PARAGRAPH)).clicked() {
                                doc.sort_works_cited();
                            }
                        });
                    });

                    ui.add_space(40.0);
                });

                if margin_left > 0.0 {
                    ui.add_space(margin_left);
                }
            });
        });

    action
}
