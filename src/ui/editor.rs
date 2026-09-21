use crate::fonts::{doc_font, icons};
use crate::keybinds::{Action, KeybindConfig};
use crate::model::format_current_mla_date;
use crate::model::{MlaBlock, MlaDocument};
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

    // Ensure blocks and body are properly populated
    doc.ensure_blocks_initialized();
    doc.ensure_body_synced();

    let avail_h = ui.available_height();
    let scroll_h = if focus_mode {
        avail_h
    } else {
        (avail_h - 34.0).max(150.0)
    };

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .max_height(scroll_h)
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
                        if let Some(act) = render_keybind_preview_panel(ui, doc, theme, keybinds, sidebar_width) {
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

                        // --- STRUCTURED MLA BODY BLOCKS EDITOR ---
                        let mut block_to_delete = None;
                        let mut block_to_move_up = None;
                        let mut block_to_move_down = None;
                        let mut block_splits: Vec<(usize, String, Vec<String>)> = Vec::new();
                        let mut focus_target_id = None;
                        let mut blocks_changed = false;

                        let total_blocks = doc.blocks.len();
                        for b_idx in 0..total_blocks {
                            if b_idx > 0 {
                                ui.add_space(14.0); // Distinct visual spacing between paragraphs & blocks!
                            }

                            let block = &mut doc.blocks[b_idx];
                            match block {
                                MlaBlock::Paragraph { id, text } => {
                                    let block_id = id.clone();
                                    let text_edit_id = egui::Id::new("p_block").with(&block_id);

                                    let resp = ui.add(
                                        egui::TextEdit::multiline(text)
                                            .id(text_edit_id)
                                            .font(doc_font(16.0))
                                            .text_color(text_col)
                                            .frame(egui::Frame::NONE)
                                            .desired_width(printable_width)
                                            .desired_rows(2)
                                            .hint_text(
                                                RichText::new("Begin typing your paragraph here...")
                                                    .italics()
                                                    .color(muted_col),
                                            ),
                                    );

                                    if resp.changed() {
                                        blocks_changed = true;
                                        if text.contains('\n') {
                                            let lines: Vec<&str> = text.split('\n').collect();
                                            if lines.len() > 1 {
                                                let first_part = lines[0].to_string();
                                                let rest_parts: Vec<String> = lines[1..]
                                                    .iter()
                                                    .map(|s| s.to_string())
                                                    .collect();
                                                block_splits.push((b_idx, first_part, rest_parts));
                                            }
                                        }
                                    }

                                    // If empty paragraph and backspace pressed, remove block (if > 1)
                                    if resp.has_focus()
                                        && text.is_empty()
                                        && total_blocks > 1
                                        && ui.input(|i| i.key_pressed(egui::Key::Backspace))
                                    {
                                        block_to_delete = Some(b_idx);
                                    }
                                }
                                MlaBlock::BlockQuote {
                                    id: _,
                                    text,
                                    citation,
                                } => {
                                    egui::Frame::new()
                                        .fill(theme.card_fill_color())
                                        .stroke(egui::Stroke::new(
                                            1.0,
                                            accent_col.gamma_multiply(0.4),
                                        ))
                                        .corner_radius(6.0)
                                        .inner_margin(egui::Margin::symmetric(14, 10))
                                        .show(ui, |ui| {
                                            ui.spacing_mut().item_spacing.y = 6.0;

                                            // Top Header bar
                                            ui.horizontal(|ui| {
                                                ui.label(
                                                    RichText::new("❝ MLA Block Quote (0.5\" Indent)")
                                                        .size(11.5)
                                                        .strong()
                                                        .color(accent_col),
                                                );

                                                ui.with_layout(
                                                    egui::Layout::right_to_left(egui::Align::Center),
                                                    |ui| {
                                                        if ui
                                                            .small_button(icons::TRASH)
                                                            .on_hover_text("Delete Block Quote")
                                                            .clicked()
                                                        {
                                                            block_to_delete = Some(b_idx);
                                                        }
                                                        if b_idx + 1 < total_blocks
                                                            && ui
                                                                .small_button("▼")
                                                                .on_hover_text("Move Down")
                                                                .clicked()
                                                        {
                                                            block_to_move_down = Some(b_idx);
                                                        }
                                                        if b_idx > 0
                                                            && ui
                                                                .small_button("▲")
                                                                .on_hover_text("Move Up")
                                                                .clicked()
                                                        {
                                                            block_to_move_up = Some(b_idx);
                                                        }
                                                    },
                                                );
                                            });

                                            // Quotation content (without markdown > prefix!)
                                            let q_resp = ui.add(
                                                egui::TextEdit::multiline(text)
                                                    .font(doc_font(15.5))
                                                    .text_color(text_col)
                                                    .desired_width(printable_width - 28.0)
                                                    .desired_rows(3)
                                                    .hint_text(
                                                        RichText::new(
                                                            "Enter quoted passage (required for prose >4 lines)...",
                                                        )
                                                        .italics()
                                                        .color(muted_col),
                                                    ),
                                            );
                                            if q_resp.changed() {
                                                blocks_changed = true;
                                            }

                                            // Parenthetical Citation
                                            ui.horizontal(|ui| {
                                                ui.label(
                                                    RichText::new("Citation:")
                                                        .size(12.0)
                                                        .color(muted_col),
                                                );
                                                let c_resp = ui.add(
                                                    egui::TextEdit::singleline(citation)
                                                        .font(doc_font(14.0))
                                                        .text_color(text_col)
                                                        .desired_width(180.0)
                                                        .hint_text("(Author 42)"),
                                                );
                                                if c_resp.changed() {
                                                    blocks_changed = true;
                                                }

                                                if ui
                                                    .small_button(format!(
                                                        "{} Pick from Works Cited",
                                                        icons::BOOK_CITATIONS
                                                    ))
                                                    .clicked()
                                                {
                                                    action = Some(EditorAction::OpenCitationModal(
                                                        Some(b_idx),
                                                    ));
                                                }
                                            });
                                        });
                                }
                                MlaBlock::SectionHeading {
                                    id: _,
                                    level,
                                    text,
                                } => {
                                    egui::Frame::new()
                                        .fill(theme.card_fill_color())
                                        .stroke(egui::Stroke::new(
                                            1.0,
                                            accent_col.gamma_multiply(0.3),
                                        ))
                                        .corner_radius(6.0)
                                        .inner_margin(egui::Margin::symmetric(14, 10))
                                        .show(ui, |ui| {
                                            ui.spacing_mut().item_spacing.y = 6.0;

                                            // Top header with level selector
                                            ui.horizontal(|ui| {
                                                ui.label(
                                                    RichText::new("Section Heading:")
                                                        .size(11.5)
                                                        .color(muted_col),
                                                );

                                                let h1_active = *level == 1;
                                                if ui
                                                    .selectable_label(h1_active, "H1 (Bold Flush Left)")
                                                    .clicked()
                                                {
                                                    *level = 1;
                                                    blocks_changed = true;
                                                }

                                                let h2_active = *level == 2;
                                                if ui
                                                    .selectable_label(
                                                        h2_active,
                                                        "H2 (Italic Flush Left)",
                                                    )
                                                    .clicked()
                                                {
                                                    *level = 2;
                                                    blocks_changed = true;
                                                }

                                                let h3_active = *level == 3;
                                                if ui
                                                    .selectable_label(h3_active, "H3 (Bold Centered)")
                                                    .clicked()
                                                {
                                                    *level = 3;
                                                    blocks_changed = true;
                                                }

                                                ui.with_layout(
                                                    egui::Layout::right_to_left(egui::Align::Center),
                                                    |ui| {
                                                        if ui
                                                            .small_button(icons::TRASH)
                                                            .on_hover_text("Delete Heading")
                                                            .clicked()
                                                        {
                                                            block_to_delete = Some(b_idx);
                                                        }
                                                        if b_idx + 1 < total_blocks
                                                            && ui
                                                                .small_button("▼")
                                                                .on_hover_text("Move Down")
                                                                .clicked()
                                                        {
                                                            block_to_move_down = Some(b_idx);
                                                        }
                                                        if b_idx > 0
                                                            && ui
                                                                .small_button("▲")
                                                                .on_hover_text("Move Up")
                                                                .clicked()
                                                        {
                                                            block_to_move_up = Some(b_idx);
                                                        }
                                                    },
                                                );
                                            });

                                            // Styled Heading Input (without markdown # prefix!)
                                            let (hint, font_style) = match *level {
                                                1 => (
                                                    "Level 1 Heading (Bold Flush Left)",
                                                    doc_font(17.0),
                                                ),
                                                2 => (
                                                    "Level 2 Heading (Italic Flush Left)",
                                                    doc_font(16.0),
                                                ),
                                                _ => (
                                                    "Level 3 Heading (Bold Centered)",
                                                    doc_font(16.0),
                                                ),
                                            };

                                            let align = if *level == 3 {
                                                egui::Align::Center
                                            } else {
                                                egui::Align::Min
                                            };

                                            let h_resp = ui.add(
                                                egui::TextEdit::singleline(text)
                                                    .font(font_style)
                                                    .text_color(text_col)
                                                    .horizontal_align(align)
                                                    .desired_width(printable_width - 28.0)
                                                    .hint_text(
                                                        RichText::new(hint)
                                                            .italics()
                                                            .color(muted_col),
                                                    ),
                                            );
                                            if h_resp.changed() {
                                                blocks_changed = true;
                                            }
                                        });
                                }
                            }
                        }

                        // Apply block splits (single Enter key splits paragraph!)
                        for (split_idx, first_part, rest_parts) in block_splits {
                            if let MlaBlock::Paragraph { text, .. } = &mut doc.blocks[split_idx] {
                                *text = first_part;
                            }
                            let mut ins_idx = split_idx;
                            for p_text in rest_parts {
                                ins_idx = doc.add_paragraph(Some(ins_idx));
                                if let MlaBlock::Paragraph { text, id } = &mut doc.blocks[ins_idx] {
                                    *text = p_text;
                                    focus_target_id =
                                        Some(egui::Id::new("p_block").with(id.clone()));
                                }
                            }
                            blocks_changed = true;
                        }

                        if let Some(target_id) = focus_target_id {
                            ui.ctx().memory_mut(|m| m.request_focus(target_id));
                        }

                        // Apply delete/reorder actions
                        if let Some(idx) = block_to_delete {
                            doc.remove_block(idx);
                            blocks_changed = true;
                        }
                        if let Some(idx) = block_to_move_up {
                            doc.move_block_up(idx);
                            blocks_changed = true;
                        }
                        if let Some(idx) = block_to_move_down {
                            doc.move_block_down(idx);
                            blocks_changed = true;
                        }

                        if blocks_changed {
                            doc.is_dirty = true;
                            doc.sync_body_from_blocks();
                        }

                        // Insert block buttons below manuscript
                        ui.add_space(16.0);
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 8.0;

                            if ui
                                .button(
                                    RichText::new(format!("{} Paragraph", icons::PLUS))
                                        .color(accent_col),
                                )
                                .on_hover_text("Add new body paragraph")
                                .clicked()
                            {
                                doc.add_paragraph(None);
                            }

                            if ui
                                .button(
                                    RichText::new(format!("{} Block Quote", icons::QUOTE))
                                        .color(text_col),
                                )
                                .on_hover_text("Insert MLA block quotation")
                                .clicked()
                            {
                                doc.add_blockquote(None);
                            }

                            if ui
                                .button(
                                    RichText::new(format!("{} Heading", icons::HEADING))
                                        .color(text_col),
                                )
                                .on_hover_text("Insert section heading")
                                .clicked()
                            {
                                doc.add_heading(1, None);
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
    doc: &MlaDocument,
    theme: &ThemeConfig,
    keybinds: &KeybindConfig,
    sidebar_width: f32,
) -> Option<Action> {
    let mut triggered = None;
    let accent_col = theme.accent_color();
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

            // --- Document Stats Section ---
            ui.label(
                RichText::new(format!("{} DOCUMENT STATS", icons::TEXT))
                    .strong()
                    .size(10.5)
                    .color(accent_col),
            );

            let word_count = doc.total_word_count();
            let est_pages = doc.estimated_page_count();
            let char_count = doc.total_char_count();
            let read_min = ((word_count as f32) / 200.0).ceil() as usize;

            ui.horizontal(|ui| {
                ui.label(RichText::new("Word Count:").size(11.0).color(muted_col));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("{}", word_count))
                            .strong()
                            .size(11.5)
                            .color(theme.text_color()),
                    );
                });
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("PDF Pages:").size(11.0).color(muted_col));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("~{}", est_pages))
                            .strong()
                            .size(11.5)
                            .color(accent_col),
                    );
                });
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Characters:").size(11.0).color(muted_col));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("{}", char_count))
                            .size(11.0)
                            .color(muted_col),
                    );
                });
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Read Time:").size(11.0).color(muted_col));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("~{} min", read_min.max(1)))
                            .size(11.0)
                            .color(muted_col),
                    );
                });
            });

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(2.0);

            // Header
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("{} Shortcuts", icons::EDIT))
                        .strong()
                        .size(12.5)
                        .color(accent_col),
                );
            });
            ui.label(
                RichText::new("Live custom keybinds")
                    .size(10.0)
                    .color(muted_col),
            );

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
