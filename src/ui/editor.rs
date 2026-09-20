use crate::model::{format_current_mla_date, to_mla_title_case, MlaBlock, MlaDocument};
use crate::theme::ThemeConfig;
use egui::{FontId, RichText, Ui};

pub enum EditorAction {
    OpenCitationModal(usize),
    OpenWorksCitedModal(Option<usize>),
    RequestRepaint,
}

pub fn render_editor_page(
    ui: &mut Ui,
    doc: &mut MlaDocument,
    theme: &ThemeConfig,
    active_block_idx: &mut Option<usize>,
) -> Option<EditorAction> {
    let mut action = None;
    let text_col = theme.text_color();
    let muted_col = theme.muted_text_color();
    let accent_col = theme.accent_color();

    // Scrollable area for document canvas
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add_space(20.0);

            // Center the simulated 8.5 x 11 inch paper sheet
            ui.horizontal(|ui| {
                let available_width = ui.available_width();
                let page_width = 820.0f32.min(available_width - 32.0);
                let side_margin = ((available_width - page_width) / 2.0).max(16.0);

                ui.add_space(side_margin);

                // Draw the translucent frosted paper sheet
                let frame = egui::Frame::new()
                    .fill(theme.page_fill_color())
                    .stroke(theme.page_stroke())
                    .corner_radius(8.0)
                    .inner_margin(egui::Margin::symmetric(54, 48)); // ~ 1-inch visual margins!

                frame.show(ui, |ui| {
                    ui.set_width(page_width - 108.0);
                    ui.spacing_mut().item_spacing.y = 10.0;

                    // --- Running Head (Top Right, 0.5 inch from top edge) ---
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                        let derived_last = doc.header.derived_last_name();
                        ui.label(
                            RichText::new(format!("{} 1", derived_last))
                                .font(FontId::new(14.0, egui::FontFamily::Proportional))
                                .color(muted_col),
                        );
                    });

                    ui.add_space(4.0);

                    // --- First Page Header Identification (Flush Left, Double Spaced) ---
                    ui.group(|ui| {
                        ui.spacing_mut().item_spacing.y = 6.0;

                        // Student Name
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Student Name:").size(11.0).color(muted_col));
                            ui.add(
                                egui::TextEdit::singleline(&mut doc.header.student_name)
                                    .hint_text("First Last")
                                    .font(FontId::new(15.0, egui::FontFamily::Proportional))
                                    .desired_width(280.0),
                            );
                        });

                        // Instructor Name
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Instructor:    ").size(11.0).color(muted_col));
                            ui.add(
                                egui::TextEdit::singleline(&mut doc.header.instructor_name)
                                    .hint_text("e.g. Professor Smith")
                                    .font(FontId::new(15.0, egui::FontFamily::Proportional))
                                    .desired_width(280.0),
                            );
                        });

                        // Course Title/Number
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Course:        ").size(11.0).color(muted_col));
                            ui.add(
                                egui::TextEdit::singleline(&mut doc.header.course)
                                    .hint_text("e.g. ENG 101: Composition")
                                    .font(FontId::new(15.0, egui::FontFamily::Proportional))
                                    .desired_width(280.0),
                            );
                        });

                        // Date (Day Month Year) with Today shortcut
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Date (MLA):    ").size(11.0).color(muted_col));
                            ui.add(
                                egui::TextEdit::singleline(&mut doc.header.date)
                                    .hint_text("e.g. 20 September 2026")
                                    .font(FontId::new(15.0, egui::FontFamily::Proportional))
                                    .desired_width(200.0),
                            );
                            if ui.button(RichText::new("📅 Today (MLA)").size(11.0)).clicked() {
                                doc.header.date = format_current_mla_date();
                                doc.is_dirty = true;
                            }
                        });
                    });

                    ui.add_space(8.0);

                    // --- Paper Title (Centered, Standard 12pt, Not Bold) ---
                    ui.vertical_centered(|ui| {
                        ui.horizontal(|ui| {
                            ui.add_space((ui.available_width() - 500.0).max(0.0) / 2.0);
                            let title_edit = ui.add(
                                egui::TextEdit::singleline(&mut doc.title)
                                    .hint_text("Centered Document Title (MLA Title Case)")
                                    .font(FontId::new(16.0, egui::FontFamily::Proportional))
                                    .desired_width(460.0),
                            );
                            if title_edit.changed() {
                                doc.is_dirty = true;
                            }
                            if ui.button(RichText::new("Aa").size(11.0)).on_hover_text("Convert to MLA Title Case").clicked() {
                                doc.title = to_mla_title_case(&doc.title);
                                doc.is_dirty = true;
                            }
                        });
                    });

                    ui.add_space(14.0);

                    // --- Body Blocks ---
                    let mut block_to_remove = None;
                    let mut block_to_move_up = None;
                    let mut block_to_move_down = None;

                    let block_count = doc.blocks.len();

                    for i in 0..block_count {
                        let block = &mut doc.blocks[i];

                        ui.push_id(i, |ui| {
                            match block {
                                MlaBlock::Paragraph { text, .. } => {
                                    ui.horizontal_top(|ui| {
                                        // MLA 0.5-inch visual first-line indent marker
                                        ui.add_space(28.0);

                                        let edit = ui.add(
                                            egui::TextEdit::multiline(text)
                                                .desired_rows(3)
                                                .desired_width(ui.available_width() - 85.0)
                                                .font(FontId::new(15.0, egui::FontFamily::Proportional)),
                                        );
                                        if edit.changed() {
                                            doc.is_dirty = true;
                                        }
                                        if edit.has_focus() {
                                            *active_block_idx = Some(i);
                                        }

                                        // Block controls on side
                                        ui.vertical(|ui| {
                                            ui.spacing_mut().item_spacing.y = 2.0;

                                            if ui.small_button("📌").on_hover_text("Insert citation here").clicked() {
                                                action = Some(EditorAction::OpenCitationModal(i));
                                            }
                                            if i > 0 && ui.small_button("▲").clicked() {
                                                block_to_move_up = Some(i);
                                            }
                                            if i + 1 < block_count && ui.small_button("▼").clicked() {
                                                block_to_move_down = Some(i);
                                            }
                                            if block_count > 1 && ui.small_button("✕").on_hover_text("Delete paragraph").clicked() {
                                                block_to_remove = Some(i);
                                            }
                                        });
                                    });
                                }
                                MlaBlock::BlockQuote { text, citation, .. } => {
                                    ui.group(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new("MLA Block Quote (> 4 lines)").size(11.0).italics().color(accent_col));
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                if ui.small_button("✕").clicked() {
                                                    block_to_remove = Some(i);
                                                }
                                                if i + 1 < block_count && ui.small_button("▼").clicked() {
                                                    block_to_move_down = Some(i);
                                                }
                                                if i > 0 && ui.small_button("▲").clicked() {
                                                    block_to_move_up = Some(i);
                                                }
                                            });
                                        });

                                        ui.horizontal_top(|ui| {
                                            // MLA 0.5-inch left indent
                                            ui.add_space(28.0);
                                            let edit = ui.add(
                                                egui::TextEdit::multiline(text)
                                                    .desired_rows(3)
                                                    .desired_width(ui.available_width() - 20.0)
                                                    .font(FontId::new(14.5, egui::FontFamily::Proportional)),
                                            );
                                            if edit.changed() {
                                                doc.is_dirty = true;
                                            }
                                        });

                                        ui.horizontal(|ui| {
                                            ui.add_space(28.0);
                                            ui.label(RichText::new("Parenthetical Citation:").size(11.0).color(muted_col));
                                            let cite_edit = ui.add(
                                                egui::TextEdit::singleline(citation)
                                                    .hint_text("e.g. (Smith 42)")
                                                    .desired_width(200.0),
                                            );
                                            if cite_edit.changed() {
                                                doc.is_dirty = true;
                                            }
                                        });
                                    });
                                }
                                MlaBlock::SectionHeading { level, text, .. } => {
                                    ui.horizontal(|ui| {
                                        let (lbl, align_center) = match level {
                                            1 => ("Level 1 (Bold)", false),
                                            2 => ("Level 2 (Italics)", false),
                                            _ => ("Level 3 (Centered Bold)", true),
                                        };

                                        ui.label(RichText::new(lbl).size(10.5).color(muted_col));

                                        if align_center {
                                            ui.add_space((ui.available_width() - 400.0).max(0.0) / 2.0);
                                        }

                                        let edit = ui.add(
                                            egui::TextEdit::singleline(text)
                                                .hint_text("Section Heading")
                                                .desired_width(380.0),
                                        );
                                        if edit.changed() {
                                            doc.is_dirty = true;
                                        }

                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            if ui.small_button("✕").clicked() {
                                                block_to_remove = Some(i);
                                            }
                                            if i + 1 < block_count && ui.small_button("▼").clicked() {
                                                block_to_move_down = Some(i);
                                            }
                                            if i > 0 && ui.small_button("▲").clicked() {
                                                block_to_move_up = Some(i);
                                            }
                                        });
                                    });
                                }
                            }
                        });

                        ui.add_space(4.0);
                    }

                    // Apply block movements/removals
                    if let Some(idx) = block_to_remove {
                        doc.remove_block(idx);
                    } else if let Some(idx) = block_to_move_up {
                        doc.move_block_up(idx);
                    } else if let Some(idx) = block_to_move_down {
                        doc.move_block_down(idx);
                    }

                    // Quick Append Block Bar
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("+ Insert:").size(12.0).color(muted_col));
                        if ui.button(RichText::new("¶ Paragraph").size(12.0)).clicked() {
                            doc.add_paragraph(None);
                        }
                        if ui.button(RichText::new("❝ Block Quote").size(12.0)).clicked() {
                            doc.add_blockquote(None);
                        }
                        if ui.button(RichText::new("H1 Section").size(12.0)).clicked() {
                            doc.add_heading(1, None);
                        }
                    });

                    // --- Works Cited Section (Simulated Page Break) ---
                    ui.add_space(24.0);
                    ui.separator();

                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new("--- Page Break (MLA Works Cited Begins on New Page) ---")
                                .size(11.5)
                                .italics()
                                .color(muted_col),
                        );
                    });

                    ui.add_space(8.0);

                    // Running head for Works Cited page
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                        let derived_last = doc.header.derived_last_name();
                        let est_pages = doc.estimated_page_count();
                        ui.label(
                            RichText::new(format!("{} {}", derived_last, est_pages))
                                .font(FontId::new(14.0, egui::FontFamily::Proportional))
                                .color(muted_col),
                        );
                    });

                    // Centered "Works Cited" / "Work Cited"
                    ui.vertical_centered(|ui| {
                        let wc_heading = if doc.works_cited.len() == 1 {
                            "Work Cited"
                        } else {
                            "Works Cited"
                        };
                        ui.label(
                            RichText::new(wc_heading)
                                .font(FontId::new(16.0, egui::FontFamily::Proportional))
                                .color(text_col),
                        );
                    });

                    ui.add_space(10.0);

                    // Works Cited Entries (with 0.5-inch Hanging Indent!)
                    if doc.works_cited.is_empty() {
                        ui.vertical_centered(|ui| {
                            ui.label(
                                RichText::new("No Works Cited entries yet. Click below to add one with MLA 9 core elements.")
                                    .size(12.0)
                                    .italics()
                                    .color(muted_col),
                            );
                        });
                    } else {
                        let mut entry_to_edit = None;
                        let mut entry_to_delete = None;

                        for (e_idx, entry) in doc.works_cited.iter().enumerate() {
                            ui.group(|ui| {
                                ui.horizontal(|ui| {
                                    // Hanging Indent: First line flush left, subsequent lines indented
                                    let formatted = entry.format_markdown();
                                    ui.label(
                                        RichText::new(format!("{}. ", e_idx + 1))
                                            .size(11.0)
                                            .color(muted_col),
                                    );
                                    ui.label(
                                        RichText::new(formatted)
                                            .font(FontId::new(14.0, egui::FontFamily::Proportional))
                                            .color(text_col),
                                    );

                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.small_button("🗑").on_hover_text("Delete entry").clicked() {
                                            entry_to_delete = Some(e_idx);
                                        }
                                        if ui.small_button("✏ Edit").clicked() {
                                            entry_to_edit = Some(e_idx);
                                        }
                                    });
                                });
                            });
                            ui.add_space(4.0);
                        }

                        if let Some(idx) = entry_to_delete {
                            doc.works_cited.remove(idx);
                            doc.is_dirty = true;
                        }
                        if let Some(idx) = entry_to_edit {
                            action = Some(EditorAction::OpenWorksCitedModal(Some(idx)));
                        }
                    }

                    // Works Cited Management buttons
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button(RichText::new("➕ Add Works Cited Entry").strong().color(accent_col)).clicked() {
                            action = Some(EditorAction::OpenWorksCitedModal(None));
                        }

                        if doc.works_cited.len() > 1 && ui.button("🔤 Sort Alphabetically (MLA Rule)").clicked() {
                            doc.sort_works_cited();
                        }
                    });

                    ui.add_space(24.0);
                });

                ui.add_space(side_margin);
            });

            ui.add_space(40.0);
        });

    action
}
