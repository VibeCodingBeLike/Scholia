use crate::fonts::{doc_font, icons};
use crate::keybinds::{Action, KeybindConfig};
use crate::model::{format_current_mla_date, MlaBlock, MlaDocument};
use crate::theme::ThemeConfig;
use egui::{RichText, Ui};

pub enum EditorAction {
    OpenCitationModal(Option<usize>),
    OpenWorksCitedModal(Option<usize>),
    OpenCalendar(egui::Pos2),
    TriggerAction(Action),
}

pub fn render_editor_page(
    ui: &mut Ui,
    doc: &mut MlaDocument,
    theme: &ThemeConfig,
    keybinds: &KeybindConfig,
    focus_mode: bool,
) -> Option<EditorAction> {
    let mut action = None;
    let text_col = theme.text_color();
    let muted_col = theme.muted_text_color();
    let accent_col = theme.accent_color();

    // Ensure blocks and body text are populated and initialized
    doc.ensure_blocks_initialized();

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add_space(20.0);

            // Compute page dimensions - the manuscript page MUST ALWAYS be strictly centered!
            let total_avail = ui.available_width();
            let page_width = 816.0f32.min(total_avail - 48.0).max(420.0);
            let page_margin_left = ((total_avail - page_width) / 2.0).max(0.0);

            let sidebar_width = 208.0f32;

            // Show keybind sidebar ONLY if the left margin is wide enough to fit it comfortably
            let show_sidebar = !focus_mode && (page_margin_left >= sidebar_width + 36.0);

            if show_sidebar {
                let sidebar_x = 24.0f32;
                egui::Area::new(egui::Id::new("editor_keybind_preview_sidebar"))
                    .fixed_pos(egui::pos2(sidebar_x, 70.0))
                    .show(ui.ctx(), |ui| {
                        if let Some(act) = render_keybind_preview_panel(ui, theme, keybinds, doc, sidebar_width) {
                            action = Some(EditorAction::TriggerAction(act));
                        }
                    });
            }

            // Center container holding strictly the manuscript page
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;

                if page_margin_left > 0.0 {
                    ui.add_space(page_margin_left);
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
                        let head_str = if derived_last.is_empty() {
                            "1".to_string()
                        } else {
                            format!("{} 1", derived_last)
                        };
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                            ui.label(
                                RichText::new(head_str)
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

                                let today_btn = ui.small_button(format!("{} Today", icons::CALENDAR))
                                    .on_hover_text("Left-click: Insert today's date\nRight-click: Open calendar date picker");

                                if today_btn.clicked() {
                                    doc.header.date = format_current_mla_date();
                                    doc.is_dirty = true;
                                }

                                if today_btn.secondary_clicked() {
                                    action = Some(EditorAction::OpenCalendar(today_btn.rect.left_bottom() + egui::vec2(0.0, 4.0)));
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
                                    .horizontal_align(egui::Align::Center)
                                    .frame(egui::Frame::NONE)
                                    .hint_text(RichText::new("Title of Your Paper (MLA Title Case)").italics().color(muted_col))
                                    .desired_width(printable_width),
                            );
                            if title_edit.changed() {
                                doc.is_dirty = true;
                            }
                        });

                        ui.add_space(20.0);

                        // --- STRUCTURED MLA BODY BLOCKS ---
                        let mut block_to_remove = None;
                        let mut block_to_move_up = None;
                        let mut block_to_move_down = None;
                        let mut block_to_split = None;
                        let mut blocks_changed = false;
                        let block_count = doc.blocks.len();

                        for (idx, block) in doc.blocks.iter_mut().enumerate() {
                            match block {
                                MlaBlock::Paragraph { text, .. } => {
                                    ui.add_space(4.0);
                                    let resp = ui.add(
                                        egui::TextEdit::multiline(text)
                                            .font(doc_font(16.0))
                                            .text_color(text_col)
                                            .frame(egui::Frame::NONE)
                                            .desired_width(printable_width)
                                            .hint_text(RichText::new("Begin typing paragraph (0.5\" first-line indent applied in MLA)...").italics().color(muted_col)),
                                    );
                                    if resp.changed() {
                                        blocks_changed = true;
                                        if let Some(newline_pos) = text.find('\n') {
                                            let next_part = text[newline_pos + 1..].to_string();
                                            text.truncate(newline_pos);
                                            block_to_split = Some((idx, next_part));
                                        }
                                    }
                                    // Visual paragraph spacing
                                    ui.add_space(12.0);
                                }
                                MlaBlock::BlockQuote { text, citation, .. } => {
                                    ui.add_space(8.0);
                                    egui::Frame::new()
                                        .fill(theme.card_fill_color())
                                        .stroke(egui::Stroke::new(1.0, accent_col.gamma_multiply(0.4)))
                                        .corner_radius(6.0)
                                        .inner_margin(egui::Margin::symmetric(14, 10))
                                        .show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                ui.label(
                                                    RichText::new(format!("{} MLA Block Quote (0.5\" Indent)", icons::QUOTE))
                                                        .size(11.5)
                                                        .strong()
                                                        .color(accent_col),
                                                );
                                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                    if ui.small_button(icons::TRASH).on_hover_text("Delete block quote").clicked() {
                                                        block_to_remove = Some(idx);
                                                    }
                                                    if idx + 1 < block_count && ui.small_button(icons::ARROW_DOWN).clicked() {
                                                        block_to_move_down = Some(idx);
                                                    }
                                                    if idx > 0 && ui.small_button(icons::ARROW_UP).clicked() {
                                                        block_to_move_up = Some(idx);
                                                    }
                                                });
                                            });
                                            ui.add_space(4.0);

                                            // 0.5-inch visual indent inside the blockquote
                                            ui.horizontal_top(|ui| {
                                                ui.add_space(20.0);
                                                let q_edit = ui.add(
                                                    egui::TextEdit::multiline(text)
                                                        .font(doc_font(16.0))
                                                        .text_color(text_col)
                                                        .frame(egui::Frame::NONE)
                                                        .hint_text(RichText::new("Quotation text (no enclosing quotation marks needed in MLA)...").italics().color(muted_col))
                                                        .desired_width(printable_width - 64.0),
                                                );
                                                if q_edit.changed() {
                                                    blocks_changed = true;
                                                }
                                            });

                                            ui.add_space(6.0);
                                            ui.horizontal(|ui| {
                                                ui.add_space(20.0);
                                                ui.label(RichText::new("Citation:").size(12.0).color(muted_col));
                                                let c_edit = ui.add(
                                                    egui::TextEdit::singleline(citation)
                                                        .font(doc_font(14.0))
                                                        .text_color(text_col)
                                                        .hint_text(RichText::new("e.g. (Smith 42)").italics().color(muted_col))
                                                        .desired_width(180.0),
                                                );
                                                if c_edit.changed() {
                                                    blocks_changed = true;
                                                }
                                            });
                                        });
                                    ui.add_space(10.0);
                                }
                                MlaBlock::SectionHeading { level, text, .. } => {
                                    ui.add_space(10.0);
                                    egui::Frame::new()
                                        .fill(theme.card_fill_color())
                                        .stroke(egui::Stroke::new(1.0, theme.page_stroke().color.gamma_multiply(0.6)))
                                        .corner_radius(6.0)
                                        .inner_margin(egui::Margin::symmetric(14, 10))
                                        .show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                let tag = match level {
                                                    1 => "H1 (Bold Flush Left)",
                                                    2 => "H2 (Italics Flush Left)",
                                                    _ => "H3 (Bold Centered)",
                                                };
                                                ui.label(
                                                    RichText::new(format!("{} {}", icons::HEADING, tag))
                                                        .size(11.0)
                                                        .strong()
                                                        .color(accent_col),
                                                );

                                                if ui.small_button(if *level == 1 { "H1 → H2" } else { "H2 → H1" }).clicked() {
                                                    *level = if *level == 1 { 2 } else { 1 };
                                                    blocks_changed = true;
                                                }

                                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                    if ui.small_button(icons::TRASH).on_hover_text("Delete heading").clicked() {
                                                        block_to_remove = Some(idx);
                                                    }
                                                    if idx + 1 < block_count && ui.small_button(icons::ARROW_DOWN).clicked() {
                                                        block_to_move_down = Some(idx);
                                                    }
                                                    if idx > 0 && ui.small_button(icons::ARROW_UP).clicked() {
                                                        block_to_move_up = Some(idx);
                                                    }
                                                });
                                            });

                                            ui.add_space(6.0);

                                            let h_edit = ui.add(
                                                egui::TextEdit::singleline(text)
                                                    .font(doc_font(16.0))
                                                    .text_color(text_col)
                                                    .frame(egui::Frame::NONE)
                                                    .hint_text(RichText::new("Section Heading Title...").italics().color(muted_col))
                                                    .desired_width(printable_width - 32.0),
                                            );
                                            if h_edit.changed() {
                                                blocks_changed = true;
                                            }
                                        });
                                    ui.add_space(10.0);
                                }
                            }
                        }

                        // Apply block mutations
                        if let Some((idx, next_part)) = block_to_split {
                            let new_idx = doc.add_paragraph(Some(idx));
                            if let Some(MlaBlock::Paragraph { text, .. }) = doc.blocks.get_mut(new_idx) {
                                *text = next_part;
                            }
                            blocks_changed = true;
                        }
                        if let Some(idx) = block_to_remove {
                            doc.remove_block(idx);
                            blocks_changed = true;
                        } else if let Some(idx) = block_to_move_up {
                            doc.move_block_up(idx);
                            blocks_changed = true;
                        } else if let Some(idx) = block_to_move_down {
                            doc.move_block_down(idx);
                            blocks_changed = true;
                        }

                        if blocks_changed {
                            doc.is_dirty = true;
                            doc.sync_body_from_blocks();
                        }

                        // Bottom Page 1 Action Bar: Quick Insert MLA Elements
                        ui.add_space(14.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("+ Insert:").size(12.0).color(muted_col));
                            if ui.small_button(format!("{} Paragraph", icons::PARAGRAPH)).clicked() {
                                doc.add_paragraph(None);
                            }
                            if ui.small_button(format!("{} Block Quote", icons::QUOTE)).clicked() {
                                doc.add_blockquote(None);
                            }
                            if ui.small_button(format!("{} Heading 1", icons::HEADING)).clicked() {
                                doc.add_heading(1, None);
                            }
                            if ui.small_button(format!("{} Heading 2", icons::HEADING)).clicked() {
                                doc.add_heading(2, None);
                            }
                        });
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
                        let wc_head = if derived_last.is_empty() {
                            format!("{}", est_pages.max(2))
                        } else {
                            format!("{} {}", derived_last, est_pages.max(2))
                        };
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                            ui.label(
                                RichText::new(wc_head)
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
            });
        });

    action
}

fn render_keybind_preview_panel(
    ui: &mut Ui,
    theme: &ThemeConfig,
    keybinds: &KeybindConfig,
    doc: &MlaDocument,
    sidebar_width: f32,
) -> Option<Action> {
    let mut triggered = None;
    let accent_col = theme.accent_color();
    let text_col = theme.text_color();
    let muted_col = theme.muted_text_color();

    egui::Frame::new()
        .fill(theme.card_fill_color())
        .stroke(theme.page_stroke())
        .corner_radius(8.0)
        .inner_margin(egui::Margin::symmetric(12, 12))
        .show(ui, |ui| {
            ui.set_width(sidebar_width);
            ui.set_min_width(sidebar_width);
            ui.set_max_width(sidebar_width);
            ui.spacing_mut().item_spacing.y = 5.0;

            // Header
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("{} Shortcuts", icons::EDIT))
                        .strong()
                        .size(13.0)
                        .color(accent_col),
                );
            });
            ui.label(
                RichText::new("Live custom keybinds")
                    .size(10.5)
                    .color(muted_col),
            );

            ui.add_space(2.0);
            ui.separator();

            // Document Stats
            ui.label(
                RichText::new("DOCUMENT STATS")
                    .size(9.5)
                    .strong()
                    .color(muted_col),
            );
            ui.horizontal(|ui| {
                ui.label(RichText::new("Words:").size(11.0).color(muted_col));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("{}", doc.total_word_count()))
                            .strong()
                            .size(11.5)
                            .color(text_col),
                    );
                });
            });
            ui.horizontal(|ui| {
                ui.label(RichText::new("Est. PDF Pages:").size(11.0).color(muted_col));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("{}", doc.estimated_page_count()))
                            .strong()
                            .size(11.5)
                            .color(text_col),
                    );
                });
            });

            ui.add_space(2.0);
            ui.separator();

            // Group 1: Writing & MLA
            ui.label(
                RichText::new("WRITING & MLA")
                    .size(9.5)
                    .strong()
                    .color(muted_col),
            );

            let actions_writing = [
                (Action::AddParagraph, icons::PARAGRAPH, "Paragraph"),
                (Action::InsertCitation, icons::QUOTE, "Citation"),
                (Action::InsertBlockQuote, icons::QUOTE, "Block Quote"),
                (
                    Action::ConvertToMlaTitleCase,
                    icons::TITLE_CASE,
                    "Title Case",
                ),
                (Action::InsertHeading1, icons::HEADING, "Heading 1"),
                (Action::InsertHeading2, icons::HEADING, "Heading 2"),
            ];

            for (act, icon, label) in actions_writing {
                let sc = keybinds.get_shortcut(act).display_string();
                if render_keybind_row(ui, theme, icon, label, &sc) {
                    triggered = Some(act);
                }
            }

            ui.add_space(3.0);
            ui.separator();

            // Group 2: File & Sources
            ui.label(
                RichText::new("FILE & SOURCES")
                    .size(9.5)
                    .strong()
                    .color(muted_col),
            );

            let actions_file = [
                (Action::SaveDocument, icons::SAVE, "Save"),
                (
                    Action::ManageWorksCited,
                    icons::BOOK_CITATIONS,
                    "Works Cited",
                ),
                (Action::ExportDocx, icons::WORD_DOCX, "Export Word"),
                (Action::ExportHtmlPdf, icons::HTML_PDF, "Export HTML"),
                (Action::NewDocument, icons::FILE_NEW, "New Paper"),
                (Action::OpenDocument, icons::FOLDER_OPEN, "Open Paper"),
            ];

            for (act, icon, label) in actions_file {
                let sc = keybinds.get_shortcut(act).display_string();
                if render_keybind_row(ui, theme, icon, label, &sc) {
                    triggered = Some(act);
                }
            }

            ui.add_space(3.0);
            ui.separator();

            // Group 3: View & Tools
            ui.label(
                RichText::new("VIEW & TOOLS")
                    .size(9.5)
                    .strong()
                    .color(muted_col),
            );

            let actions_tools = [
                (Action::ToggleFocusMode, icons::FOCUS_MODE, "Zen Mode"),
                (Action::ToggleComplianceCheck, icons::CHECK, "MLA Linter"),
                (Action::OpenPreferences, icons::SETTINGS, "Preferences"),
            ];

            for (act, icon, label) in actions_tools {
                let sc = keybinds.get_shortcut(act).display_string();
                if render_keybind_row(ui, theme, icon, label, &sc) {
                    triggered = Some(act);
                }
            }
        });

    triggered
}

fn render_keybind_row(ui: &mut Ui, theme: &ThemeConfig, icon: &str, label: &str, sc: &str) -> bool {
    let mut clicked = false;
    let text_col = theme.text_color();

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        let btn = ui.add(
            egui::Button::new(
                RichText::new(format!("{} {}", icon, label))
                    .size(11.0)
                    .color(text_col),
            )
            .frame(false),
        );
        if btn.clicked() {
            clicked = true;
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let badge = egui::Frame::new()
                .fill(egui::Color32::from_black_alpha(45))
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_white_alpha(30)))
                .corner_radius(4.0)
                .inner_margin(egui::Margin::symmetric(4, 2));

            let badge_resp = badge.show(ui, |ui| {
                ui.label(
                    RichText::new(sc)
                        .size(10.0)
                        .monospace()
                        .color(theme.accent_color())
                        .strong(),
                );
            });
            if badge_resp.response.interact(egui::Sense::click()).clicked() {
                clicked = true;
            }
        });
    });

    clicked
}
