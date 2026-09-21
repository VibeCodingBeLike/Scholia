use crate::fonts::{doc_font, icons};
use crate::keybinds::{Action, KeybindConfig};
use crate::model::format_current_mla_date;
use crate::model::{num_to_superscript, MlaBlock, MlaDocument};
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
    let status_bar_h = if focus_mode { 0.0 } else { 24.0 };
    let scroll_h = (avail_h - status_bar_h).max(100.0);

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
            let gap_to_page = 14.0f32;

            // Show keybind sidebar ONLY if enabled in settings, not in focus mode, and left margin can fit it
            let show_sidebar = theme.show_shortcuts_panel
                && !focus_mode
                && (page_margin_left >= sidebar_width + gap_to_page + 10.0);

            // Center container holding strictly the manuscript page and outside shortcuts panel
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;

                if show_sidebar {
                    let left_empty = page_margin_left - sidebar_width - gap_to_page;
                    if left_empty > 0.0 {
                        ui.add_space(left_empty);
                    }

                    ui.vertical(|ui| {
                        ui.set_width(sidebar_width);
                        if let Some(act) = render_keybind_preview_panel(ui, doc, theme, keybinds, sidebar_width) {
                            action = Some(EditorAction::TriggerAction(act));
                        }
                    });

                    ui.add_space(gap_to_page);
                } else if page_margin_left > 0.0 {
                    ui.add_space(page_margin_left);
                }

                // Vertical column holding the pages (strictly centered!)
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

                        // --- RUNNING HEAD (Top-Right: [LastName] 1) ---
                        let derived_last = doc.header.derived_last_name();
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                            ui.label(
                                RichText::new("1")
                                    .font(doc_font(15.0))
                                    .color(muted_col),
                            );
                            ui.add_space(4.0);
                            let rh_edit = ui.add(
                                egui::TextEdit::singleline(&mut doc.header.running_header_last_name)
                                    .font(doc_font(15.0))
                                    .text_color(text_col)
                                    .frame(egui::Frame::NONE)
                                    .hint_text(
                                        RichText::new(if derived_last.is_empty() { "LastName" } else { &derived_last })
                                            .italics()
                                            .color(muted_col),
                                    )
                                    .desired_width(120.0),
                            ).on_hover_text("Running Head: Student's Last Name (automatically derived, or type here to override)");
                            if rh_edit.changed() {
                                doc.is_dirty = true;
                            }
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
                                let font_id = doc_font(16.0);
                                let display_str = if doc.header.date.is_empty() {
                                    "Date (e.g. 20 September 2026)"
                                } else {
                                    &doc.header.date
                                };
                                let text_w = ui.painter().layout_no_wrap(display_str.to_string(), font_id.clone(), text_col).size().x;
                                let date_w = (text_w + 10.0).max(60.0);

                                let d_name = ui.add(
                                    egui::TextEdit::singleline(&mut doc.header.date)
                                        .font(font_id)
                                        .text_color(text_col)
                                        .frame(egui::Frame::NONE)
                                        .hint_text(RichText::new("Date (e.g. 20 September 2026)").italics().color(muted_col))
                                        .desired_width(date_w),
                                );
                                if d_name.changed() {
                                    doc.is_dirty = true;
                                }

                                ui.add_space(16.0);

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
                        let mut block_to_toggle = None;
                        let mut block_splits: Vec<(usize, String, Vec<String>)> = Vec::new();
                        let mut focus_target_id = None;
                        let mut focus_navigate: Option<(usize, bool)> = None;
                        let mut blocks_changed = false;
                        let mut note_to_delete_id: Option<String> = None;
                        let mut note_to_add_block_id: Option<String> = None;
                        let mut note_shortcut_to_link: Option<(String, usize, String)> = None;

                        // Pre-group notes by block_id for visual superscript chip rendering
                        let mut notes_by_block: std::collections::HashMap<String, Vec<(String, usize, String, String)>> = std::collections::HashMap::new();
                        for note in &doc.notes {
                            notes_by_block
                                .entry(note.block_id.clone())
                                .or_default()
                                .push((note.id.clone(), note.index, note.text.clone(), note.word.clone()));
                        }

                        let total_blocks = doc.blocks.len();
                        for b_idx in 0..total_blocks {
                            if b_idx > 0 {
                                ui.add_space(14.0); // Distinct visual spacing between paragraphs & blocks!
                            }

                            let block = &mut doc.blocks[b_idx];
                            match block {
                                MlaBlock::Paragraph { id, text, note_tags } => {
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
                                                RichText::new("Begin typing your paragraph here... (type word^(1) + space for note, /cite for citation)")
                                                    .italics()
                                                    .color(muted_col),
                                            ),
                                    );

                                    if resp.changed() {
                                        blocks_changed = true;
                                        // Note shorthand: word^(N) followed by spacebar confirmation
                                        if let Some(res) = crate::model::try_parse_and_apply_note_shortcut(text, note_tags) {
                                            note_shortcut_to_link = Some((block_id.clone(), res.note_index, res.word));
                                        }

                                        // Slash command: /cite
                                        if text.contains("/cite") {
                                            *text = text.replace("/cite", "").trim_end().to_string();
                                            action = Some(EditorAction::OpenCitationModal(Some(b_idx)));
                                        }

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

                                    // Automatic Block Quote Detection (>4 lines of prose or >250 chars)
                                    let line_count = text.lines().count();
                                    let char_count = text.len();
                                    if line_count >= 4 || char_count >= 250 {
                                        ui.horizontal(|ui| {
                                            ui.label(
                                                RichText::new("💡 MLA Rule: Quotation exceeds 4 lines of prose.")
                                                    .size(11.0)
                                                    .color(egui::Color32::from_rgb(220, 150, 30)),
                                            );
                                            if ui.small_button("Convert to Block Quote").clicked() {
                                                block_to_toggle = Some(b_idx);
                                            }
                                        });
                                    }

                                    if resp.has_focus() {
                                        doc.active_block_idx = b_idx;
                                        // If empty paragraph and backspace pressed, remove block (if > 1)
                                        if text.is_empty()
                                            && total_blocks > 1
                                            && ui.input(|i| i.key_pressed(egui::Key::Backspace))
                                        {
                                            block_to_delete = Some(b_idx);
                                        }

                                        // Arrow navigation across paragraph boxes if cursor is at top/bottom
                                        let cursor_idx = egui::text_edit::TextEditState::load(ui.ctx(), text_edit_id)
                                            .and_then(|s| s.cursor.char_range())
                                            .map(|r| r.primary.index.0)
                                            .unwrap_or(0);
                                        let is_at_top = cursor_idx == 0 || text.is_empty();
                                        let is_at_bottom = cursor_idx >= text.len() || text.is_empty();

                                        if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) && is_at_top && b_idx > 0 {
                                            focus_navigate = Some((b_idx - 1, true));
                                        } else if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) && is_at_bottom && b_idx + 1 < total_blocks {
                                            focus_navigate = Some((b_idx + 1, false));
                                        }
                                    }

                                    // Visual note chips attached to this paragraph (purely visual in editor, linked directly to definition)
                                    if let Some(linked_notes) = notes_by_block.get(&block_id) {
                                        if !linked_notes.is_empty() {
                                            ui.add_space(2.0);
                                            ui.horizontal_wrapped(|ui| {
                                                ui.add_space(36.0); // Indent to align with paragraph text
                                                for (note_id, note_idx, note_text, note_word) in linked_notes {
                                                    let sup = num_to_superscript(*note_idx);
                                                    let label = if note_word.is_empty() {
                                                        format!("Note {}", sup)
                                                    } else {
                                                        format!("Note {} on “{}”", sup, note_word)
                                                    };
                                                    let preview = if note_text.trim().is_empty() {
                                                        "empty definition (click to write on Notes page)".to_string()
                                                    } else {
                                                        let mut s = note_text.trim().replace('\n', " ");
                                                        if s.chars().count() > 28 {
                                                            s = s.chars().take(26).collect::<String>() + "...";
                                                        }
                                                        s
                                                    };

                                                    egui::Frame::new()
                                                        .fill(theme.card_fill_color())
                                                        .stroke(egui::Stroke::new(1.0, accent_col.gamma_multiply(0.45)))
                                                        .corner_radius(4.0)
                                                        .inner_margin(egui::Margin::symmetric(6, 2))
                                                        .show(ui, |ui| {
                                                            ui.horizontal(|ui| {
                                                                ui.label(
                                                                    RichText::new(label)
                                                                        .strong()
                                                                        .size(12.0)
                                                                        .color(accent_col),
                                                                );
                                                                ui.label(
                                                                    RichText::new(format!(": “{}”", preview))
                                                                        .italics()
                                                                        .size(11.0)
                                                                        .color(text_col),
                                                                )
                                                                .on_hover_text(format!(
                                                                    "Explanatory Note {}: {}\nClick to edit on Notes page.\nClick ✕ to delete note.",
                                                                    note_idx, note_text
                                                                ));

                                                                if ui.small_button(RichText::new("✕").size(9.0).color(muted_col))
                                                                    .on_hover_text(format!("Delete Note {} and definition", note_idx))
                                                                    .clicked()
                                                                {
                                                                    note_to_delete_id = Some(note_id.clone());
                                                                }
                                                            });
                                                        });
                                                }

                                                if ui.small_button(RichText::new("+ Note").size(10.0).color(muted_col))
                                                    .on_hover_text("Add another note to this paragraph")
                                                    .clicked()
                                                {
                                                    note_to_add_block_id = Some(block_id.clone());
                                                }
                                            });
                                        }
                                    }
                                }
                                MlaBlock::BlockQuote {
                                    id,
                                    text,
                                    citation,
                                    note_tags,
                                } => {
                                    let block_id = id.clone();
                                    let q_edit_id = egui::Id::new("bq_block").with(&block_id);
                                    let c_edit_id = egui::Id::new("bqc_block").with(&block_id);

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

                                            // Header label with toggle back to Paragraph
                                            ui.horizontal(|ui| {
                                                ui.label(
                                                    RichText::new("❝ MLA Block Quote (0.5\" Indent)")
                                                        .size(11.5)
                                                        .strong()
                                                        .color(accent_col),
                                                );
                                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                    if ui.small_button("Convert to Paragraph").clicked() {
                                                        block_to_toggle = Some(b_idx);
                                                    }
                                                });
                                            });

                                            // Quotation content (without markdown > prefix)
                                            let q_resp = ui.add(
                                                egui::TextEdit::multiline(text)
                                                    .id(q_edit_id)
                                                    .font(doc_font(15.5))
                                                    .text_color(text_col)
                                                    .desired_width(printable_width - 28.0)
                                                    .desired_rows(3)
                                                    .hint_text(
                                                        RichText::new(
                                                            "Enter quoted passage (required for prose >4 lines)... (type word^(1) + space for note, /cite for citation)",
                                                        )
                                                        .italics()
                                                        .color(muted_col),
                                                    ),
                                            );
                                            if q_resp.changed() {
                                                blocks_changed = true;
                                                if let Some(res) = crate::model::try_parse_and_apply_note_shortcut(text, note_tags) {
                                                    note_shortcut_to_link = Some((block_id.clone(), res.note_index, res.word));
                                                }
                                                if text.contains("/cite") {
                                                    *text = text.replace("/cite", "").trim_end().to_string();
                                                    action = Some(EditorAction::OpenCitationModal(Some(b_idx)));
                                                }
                                            }
                                            if q_resp.has_focus() {
                                                doc.active_block_idx = b_idx;
                                                if text.is_empty()
                                                    && citation.is_empty()
                                                    && total_blocks > 1
                                                    && ui.input(|i| i.key_pressed(egui::Key::Backspace))
                                                {
                                                    block_to_delete = Some(b_idx);
                                                }

                                                let cursor_idx = egui::text_edit::TextEditState::load(ui.ctx(), q_edit_id)
                                                    .and_then(|s| s.cursor.char_range())
                                                    .map(|r| r.primary.index.0)
                                                    .unwrap_or(0);
                                                let is_at_top = cursor_idx == 0 || text.is_empty();
                                                let is_at_bottom = cursor_idx >= text.len() || text.is_empty();

                                                if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) && is_at_top && b_idx > 0 {
                                                    focus_navigate = Some((b_idx - 1, true));
                                                } else if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) && is_at_bottom && b_idx + 1 < total_blocks {
                                                    focus_navigate = Some((b_idx + 1, false));
                                                }
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
                                                        .id(c_edit_id)
                                                        .font(doc_font(14.0))
                                                        .text_color(text_col)
                                                        .desired_width(180.0)
                                                        .hint_text("(Author 42)"),
                                                );
                                                if c_resp.changed() {
                                                    blocks_changed = true;
                                                }
                                                if c_resp.has_focus() {
                                                    doc.active_block_idx = b_idx;
                                                }
                                            });

                                            // Visual note chips attached to this blockquote
                                            if let Some(linked_notes) = notes_by_block.get(&block_id) {
                                                if !linked_notes.is_empty() {
                                                    ui.horizontal_wrapped(|ui| {
                                                        for (note_id, note_idx, note_text, note_word) in linked_notes {
                                                            let sup = num_to_superscript(*note_idx);
                                                            let label = if note_word.is_empty() {
                                                                format!("Note {}", sup)
                                                            } else {
                                                                format!("Note {} on “{}”", sup, note_word)
                                                            };
                                                            let preview = if note_text.trim().is_empty() {
                                                                "empty definition".to_string()
                                                            } else {
                                                                let mut s = note_text.trim().replace('\n', " ");
                                                                if s.chars().count() > 28 {
                                                                    s = s.chars().take(26).collect::<String>() + "...";
                                                                }
                                                                s
                                                            };

                                                            egui::Frame::new()
                                                                .fill(theme.page_fill_color())
                                                                .stroke(egui::Stroke::new(1.0, accent_col.gamma_multiply(0.45)))
                                                                .corner_radius(4.0)
                                                                .inner_margin(egui::Margin::symmetric(6, 2))
                                                                .show(ui, |ui| {
                                                                    ui.horizontal(|ui| {
                                                                        ui.label(
                                                                            RichText::new(label)
                                                                                .strong()
                                                                                .size(11.5)
                                                                                .color(accent_col),
                                                                        );
                                                                        ui.label(
                                                                            RichText::new(format!(": “{}”", preview))
                                                                                .italics()
                                                                                .size(10.5)
                                                                                .color(text_col),
                                                                        );
                                                                        if ui.small_button(RichText::new("✕").size(9.0).color(muted_col))
                                                                            .on_hover_text(format!("Delete Note {} and definition", note_idx))
                                                                            .clicked()
                                                                        {
                                                                            note_to_delete_id = Some(note_id.clone());
                                                                        }
                                                                    });
                                                                });
                                                        }
                                                    });
                                                }
                                            }
                                        });
                                }
                                MlaBlock::SectionHeading {
                                    id,
                                    level,
                                    text,
                                } => {
                                    let block_id = id.clone();
                                    let h_edit_id = egui::Id::new("h_block").with(&block_id);

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

                                            let (level_title, hint, font_style) = match *level {
                                                1 => (
                                                    "§ Section Heading 1 (Bold Flush Left)",
                                                    "Level 1 Heading (Bold Flush Left)",
                                                    doc_font(17.0),
                                                ),
                                                2 => (
                                                    "§ Section Heading 2 (Italic Flush Left)",
                                                    "Level 2 Heading (Italic Flush Left)",
                                                    doc_font(16.0),
                                                ),
                                                _ => (
                                                    "§ Section Heading 3 (Bold Centered)",
                                                    "Level 3 Heading (Bold Centered)",
                                                    doc_font(16.0),
                                                ),
                                            };

                                            ui.label(
                                                RichText::new(level_title)
                                                    .size(11.5)
                                                    .color(muted_col),
                                            );

                                            let align = if *level == 3 {
                                                egui::Align::Center
                                            } else {
                                                egui::Align::Min
                                            };

                                            let h_resp = ui.add(
                                                egui::TextEdit::singleline(text)
                                                    .id(h_edit_id)
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
                                            if h_resp.has_focus() {
                                                doc.active_block_idx = b_idx;
                                                if text.is_empty()
                                                    && total_blocks > 1
                                                    && ui.input(|i| i.key_pressed(egui::Key::Backspace))
                                                {
                                                    block_to_delete = Some(b_idx);
                                                }

                                                let cursor_idx = egui::text_edit::TextEditState::load(ui.ctx(), h_edit_id)
                                                    .and_then(|s| s.cursor.char_range())
                                                    .map(|r| r.primary.index.0)
                                                    .unwrap_or(0);
                                                let is_at_top = cursor_idx == 0 || text.is_empty();
                                                let is_at_bottom = cursor_idx >= text.len() || text.is_empty();

                                                if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) && is_at_top && b_idx > 0 {
                                                    focus_navigate = Some((b_idx - 1, true));
                                                } else if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) && is_at_bottom && b_idx + 1 < total_blocks {
                                                    focus_navigate = Some((b_idx + 1, false));
                                                }
                                            }
                                        });
                                }
                            }
                        }

                        // Apply arrow navigation between blocks
                        if let Some((target_idx, to_bottom)) = focus_navigate {
                            if let Some(target_block) = doc.blocks.get(target_idx) {
                                let (target_id, text_len) = match target_block {
                                    MlaBlock::Paragraph { id, text, .. } => (egui::Id::new("p_block").with(id), text.len()),
                                    MlaBlock::BlockQuote { id, text, .. } => (egui::Id::new("bq_block").with(id), text.len()),
                                    MlaBlock::SectionHeading { id, text, .. } => (egui::Id::new("h_block").with(id), text.len()),
                                };
                                let mut state = egui::text_edit::TextEditState::load(ui.ctx(), target_id).unwrap_or_default();
                                let target_cursor = if to_bottom { text_len } else { 0 };
                                state.cursor.set_char_range(Some(egui::text::CCursorRange::one(egui::text::CCursor::new(target_cursor))));
                                state.store(ui.ctx(), target_id);
                                ui.ctx().memory_mut(|m| m.request_focus(target_id));
                                doc.active_block_idx = target_idx;
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
                                if let MlaBlock::Paragraph { text, id, .. } = &mut doc.blocks[ins_idx] {
                                    *text = p_text;
                                    doc.active_block_idx = ins_idx;
                                    focus_target_id =
                                        Some(egui::Id::new("p_block").with(id.clone()));
                                }
                            }
                            blocks_changed = true;
                        }

                        if let Some(target_id) = focus_target_id {
                            ui.ctx().memory_mut(|m| m.request_focus(target_id));
                        }

                        // Apply block toggle actions
                        if let Some(idx) = block_to_toggle {
                            doc.toggle_block_type(idx);
                            blocks_changed = true;
                        }

                        // Apply delete actions
                        if let Some(idx) = block_to_delete {
                            let prev_idx = if idx > 0 { idx - 1 } else { 0 };
                            doc.remove_block(idx);
                            doc.active_block_idx = prev_idx;
                            doc.requested_focus_block_idx = Some(prev_idx);
                            blocks_changed = true;
                        }

                        if let Some(id_to_del) = note_to_delete_id {
                            doc.delete_note_by_id(&id_to_del);
                            blocks_changed = true;
                        }

                        if let Some(target_b_id) = note_to_add_block_id {
                            doc.add_note_to_block(&target_b_id, String::new());
                            blocks_changed = true;
                        }

                        if let Some((b_id, n_idx, word)) = note_shortcut_to_link {
                            doc.link_note_from_shortcut(&b_id, n_idx, &word);
                            blocks_changed = true;
                        }

                        if blocks_changed {
                            doc.is_dirty = true;
                            doc.sync_body_from_blocks();
                            doc.sync_notes_with_body();
                        }

                        // Focus requested block (e.g. after deletion or insertion)
                        if let Some(target_idx) = doc.requested_focus_block_idx.take() {
                            if let Some(target_block) = doc.blocks.get(target_idx) {
                                let target_id = match target_block {
                                    MlaBlock::Paragraph { id, .. } => egui::Id::new("p_block").with(id),
                                    MlaBlock::BlockQuote { id, .. } => egui::Id::new("bq_block").with(id),
                                    MlaBlock::SectionHeading { id, .. } => egui::Id::new("h_block").with(id),
                                };
                                ui.ctx().memory_mut(|m| m.request_focus(target_id));
                            }
                        }

                    });

                    // ==========================================
                    // 📝 SEPARATE PAGE: NOTES (Only present when notes exist)
                    // ==========================================
                    if !doc.notes.is_empty() {
                        ui.add_space(32.0);
                        ui.horizontal(|ui| {
                            ui.add_space((page_width - 240.0) / 2.0);
                            ui.label(
                                RichText::new("────── Page Break: Notes ──────")
                                    .size(11.5)
                                    .italics()
                                    .color(muted_col),
                            );
                        });
                        ui.add_space(12.0);

                        let notes_frame = egui::Frame::new()
                            .fill(theme.page_fill_color())
                            .stroke(theme.page_stroke())
                            .corner_radius(6.0)
                            .inner_margin(egui::Margin::symmetric(page_margin, page_margin));

                        notes_frame.show(ui, |ui| {
                            ui.set_width(printable_width);
                            ui.set_min_width(printable_width);
                            ui.set_max_width(printable_width);
                            ui.spacing_mut().item_spacing.y = 8.0;

                            // Running head for Notes
                            let derived_last = doc.header.derived_last_name();
                            let head_str = if derived_last.is_empty() {
                                "2".to_string()
                            } else {
                                format!("{} 2", derived_last)
                            };
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                                ui.label(
                                    RichText::new(head_str)
                                        .font(doc_font(15.0))
                                        .color(muted_col),
                                );
                            });

                            ui.add_space(6.0);
                            ui.vertical_centered(|ui| {
                                ui.label(
                                    RichText::new("Notes")
                                        .font(doc_font(16.0))
                                        .color(text_col),
                                );
                            });
                            ui.add_space(14.0);

                            let mut note_to_delete = None;
                            for note in &mut doc.notes {
                                ui.horizontal_top(|ui| {
                                    ui.label(
                                        RichText::new(format!("{}.", note.index))
                                            .strong()
                                            .font(doc_font(15.0))
                                            .color(accent_col),
                                    );
                                    if !note.word.is_empty() {
                                        ui.label(
                                            RichText::new(format!("“{}”:", note.word))
                                                .italics()
                                                .font(doc_font(14.5))
                                                .color(accent_col.gamma_multiply(0.85)),
                                        );
                                    }
                                    let n_edit = ui.add(
                                        egui::TextEdit::multiline(&mut note.text)
                                            .font(doc_font(15.0))
                                            .text_color(text_col)
                                            .frame(egui::Frame::NONE)
                                            .desired_width(printable_width - 80.0)
                                            .desired_rows(1)
                                            .hint_text(RichText::new("Enter explanatory note, definition, or translation...").italics().color(muted_col)),
                                    );
                                    if n_edit.changed() {
                                        doc.is_dirty = true;
                                    }

                                    if ui.small_button(RichText::new(icons::TRASH).color(muted_col))
                                        .on_hover_text(format!("Delete Note {}", note.index))
                                        .clicked()
                                    {
                                        note_to_delete = Some(note.index);
                                    }
                                });
                                ui.add_space(6.0);
                            }

                            if let Some(idx) = note_to_delete {
                                doc.delete_explanatory_note(idx);
                            }

                            ui.add_space(8.0);
                            let fn_sc = keybinds.get_shortcut(Action::AddFootnote).display_string();
                            if ui.small_button(format!("+ Add Note / Definition [{}]", fn_sc)).clicked() {
                                action = Some(EditorAction::TriggerAction(Action::AddFootnote));
                            }
                        });
                    }

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
                                    RichText::new("No entries yet")
                                        .font(doc_font(14.0))
                                        .italics()
                                        .color(muted_col),
                                );
                            });
                        } else {
                            let mut entry_to_edit = None;

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

                                    let entry_lbl = ui.add(
                                        egui::Label::new(
                                            RichText::new(formatted)
                                                .font(doc_font(15.0))
                                                .color(text_col),
                                        )
                                        .wrap()
                                        .sense(egui::Sense::click()),
                                    ).on_hover_text("Click to edit or delete entry in Works Cited manager");

                                    if entry_lbl.clicked() {
                                        entry_to_edit = Some(idx);
                                    }
                                });

                                ui.add_space(6.0);
                            }

                            if let Some(idx) = entry_to_edit {
                                action = Some(EditorAction::OpenWorksCitedModal(Some(idx)));
                            }
                        }
                    });

                    ui.add_space(40.0);
                });
            });
        });

    action
}

fn render_keybind_preview_panel(
    ui: &mut Ui,
    _doc: &MlaDocument,
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

            // Header
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("{} Shortcuts", icons::EDIT))
                        .strong()
                        .size(12.5)
                        .color(accent_col),
                );
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
                (Action::InsertCitation, icons::QUOTE, "Citation"),
                (Action::InsertBlockQuote, icons::QUOTE, "Block Quote"),
                (
                    Action::ConvertToMlaTitleCase,
                    icons::TITLE_CASE,
                    "Title Case",
                ),
                (Action::InsertHeading1, icons::HEADING, "Heading 1"),
                (Action::InsertHeading2, icons::HEADING, "Heading 2"),
                (Action::InsertHeading3, icons::HEADING, "Heading 3"),
                (Action::DeleteBlock, icons::TRASH, "Delete Block"),
                (Action::MoveBlockUp, icons::ARROW_UP, "Move Up"),
                (Action::MoveBlockDown, icons::ARROW_DOWN, "Move Down"),
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
                (
                    Action::AddFootnote,
                    icons::INFO,
                    "Notes",
                ),
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
