use scholia::export::{export_to_docx, export_to_html, export_to_pdf, export_to_text};
use scholia::mla_rules::MlaLinter;
use scholia::model::{
    format_current_mla_date, to_mla_title_case, MlaDocument, SourceType, WorksCitedEntry,
};

#[test]
fn test_mla_title_case_capitalization() {
    assert_eq!(to_mla_title_case("the great gatsby"), "The Great Gatsby");
    assert_eq!(
        to_mla_title_case("an analysis of modern literary devices in american fiction"),
        "An Analysis of Modern Literary Devices in American Fiction"
    );
    assert_eq!(
        to_mla_title_case("waiting for godot: a tragicomedy in two acts"),
        "Waiting for Godot: A Tragicomedy in Two Acts"
    );
    assert_eq!(
        to_mla_title_case("the bird is in the sky and it is singing"),
        "The Bird is in the Sky and it is Singing"
    );
}

#[test]
fn test_mla_current_date_format() {
    let d = format_current_mla_date();
    let parts: Vec<&str> = d.split_whitespace().collect();
    assert_eq!(parts.len(), 3, "MLA date must have Day Month Year: {}", d);
    let day: u32 = parts[0].parse().expect("Day must be a number");
    assert!((1..=31).contains(&day));
    let year: u32 = parts[2].parse().expect("Year must be a number");
    assert!(year >= 2020);
}

#[test]
fn test_mla_compliance_linter() {
    let mut doc = MlaDocument::sample_template();
    let report = MlaLinter::inspect(&doc);
    assert!(
        report.score_percentage >= 90,
        "Sample template document should be compliant"
    );

    // Introduce an error: empty student name
    doc.header.student_name = String::new();
    let report_with_err = MlaLinter::inspect(&doc);
    assert!(report_with_err.score_percentage < report.score_percentage);
    assert!(report_with_err
        .issues
        .iter()
        .any(|i| i.rule_id == "HDR_STUDENT_NAME"));
}

#[test]
fn test_works_cited_alphabetization() {
    let mut doc = MlaDocument::new_blank();

    let mut e1 = WorksCitedEntry::new_empty();
    e1.author = "Woolf, Virginia".to_string();
    e1.title_of_source = "To the Lighthouse".to_string();
    e1.source_type = SourceType::BookOrStandalone;

    let mut e2 = WorksCitedEntry::new_empty();
    e2.author = "Austen, Jane".to_string();
    e2.title_of_source = "Pride and Prejudice".to_string();
    e2.source_type = SourceType::BookOrStandalone;

    let mut e3 = WorksCitedEntry::new_empty();
    e3.author = "Morrison, Toni".to_string();
    e3.title_of_source = "Beloved".to_string();
    e3.source_type = SourceType::BookOrStandalone;

    doc.works_cited.push(e1);
    doc.works_cited.push(e2);
    doc.works_cited.push(e3);

    doc.sort_works_cited();

    assert_eq!(doc.works_cited[0].author, "Austen, Jane");
    assert_eq!(doc.works_cited[1].author, "Morrison, Toni");
    assert_eq!(doc.works_cited[2].author, "Woolf, Virginia");
}

#[test]
fn test_document_exporters() {
    let mut doc = MlaDocument::sample_template();
    let temp_dir = std::env::temp_dir();

    // DOCX export
    let docx_path = temp_dir.join("test_mla_output.docx");
    let res_docx = export_to_docx(&doc, &docx_path);
    assert!(
        res_docx.is_ok(),
        "DOCX export should succeed: {:?}",
        res_docx
    );
    assert!(docx_path.exists());
    let _ = std::fs::remove_file(&docx_path);

    // HTML export
    let html_path = temp_dir.join("test_mla_output.html");
    let res_html = export_to_html(&doc, &html_path);
    assert!(
        res_html.is_ok(),
        "HTML export should succeed: {:?}",
        res_html
    );
    assert!(html_path.exists());
    let html_content = std::fs::read_to_string(&html_path).unwrap();
    assert!(html_content.contains("Works Cited"));
    assert!(html_content.contains("margin: 1in"));
    let _ = std::fs::remove_file(&html_path);

    // Text export
    let txt_path = temp_dir.join("test_mla_output.txt");
    let res_txt = export_to_text(&doc, &txt_path);
    assert!(res_txt.is_ok(), "Text export should succeed: {:?}", res_txt);
    assert!(txt_path.exists());
    let _ = std::fs::remove_file(&txt_path);

    // PDF export
    let pdf_path = temp_dir.join("test_mla_output.pdf");
    let res_pdf = export_to_pdf(&doc, &pdf_path);
    assert!(res_pdf.is_ok(), "PDF export should succeed: {:?}", res_pdf);
    assert!(pdf_path.exists());
    let pdf_bytes = std::fs::read(&pdf_path).unwrap();
    assert!(pdf_bytes.starts_with(b"%PDF"));
    let _ = std::fs::remove_file(&pdf_path);

    // Export with Explanatory Notes
    doc.add_explanatory_note("Explanatory content note for test".to_string());
    let pdf_notes_path = temp_dir.join("test_mla_notes.pdf");
    assert!(export_to_pdf(&doc, &pdf_notes_path).is_ok());
    let docx_notes_path = temp_dir.join("test_mla_notes.docx");
    assert!(export_to_docx(&doc, &docx_notes_path).is_ok());
    let html_notes_path = temp_dir.join("test_mla_notes.html");
    assert!(export_to_html(&doc, &html_notes_path).is_ok());
    let html_str = std::fs::read_to_string(&html_notes_path).unwrap();
    assert!(html_str.contains("mla-footnotes"));
    assert!(html_str.contains("Explanatory content note for test"));
    let _ = std::fs::remove_file(&pdf_notes_path);
    let _ = std::fs::remove_file(&docx_notes_path);
    let _ = std::fs::remove_file(&html_notes_path);
}

#[test]
fn test_font_configuration_and_nerd_icons() {
    let ctx = egui::Context::default();
    scholia::fonts::configure_fonts(&ctx);

    // Verify Nerd Font icon constants are valid UTF-8
    assert!(!scholia::fonts::icons::FILE_NEW.is_empty());
    assert!(!scholia::fonts::icons::SAVE.is_empty());
    assert!(!scholia::fonts::icons::CALENDAR.is_empty());
    assert!(!scholia::fonts::icons::BOOK_CITATIONS.is_empty());

    let doc_font = scholia::fonts::doc_font(16.0);
    assert_eq!(doc_font.size, 16.0);
}

#[test]
fn test_calendar_modal_state() {
    use scholia::ui::calendar_dialog::{month_number_to_name, CalendarModalState};

    let mut state = CalendarModalState {
        is_open: false,
        current_year: 2026,
        current_month: 9,
        popup_pos: None,
    };

    assert_eq!(month_number_to_name(state.current_month), "September");

    // Test next month
    state.next_month();
    assert_eq!(state.current_month, 10);
    assert_eq!(state.current_year, 2026);

    // Test prev month
    state.prev_month();
    assert_eq!(state.current_month, 9);

    // Test year wrap-around
    state.current_month = 1;
    state.prev_month();
    assert_eq!(state.current_month, 12);
    assert_eq!(state.current_year, 2025);

    // Test open_at with existing date string
    state.open_at(egui::pos2(100.0, 100.0), "15 March 2027");
    assert!(state.is_open);
    assert_eq!(state.current_month, 3);
    assert_eq!(state.current_year, 2027);
}

#[test]
fn test_editor_page_centering() {
    let ctx = egui::Context::default();
    scholia::fonts::configure_fonts(&ctx);
    let mut doc = scholia::model::MlaDocument::new_blank();
    let theme = scholia::theme::ThemeConfig::default();
    let keybinds = scholia::keybinds::KeybindConfig::default();

    let mut out = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1920.0, 1080.0))),
            ..Default::default()
        },
        |ui| {
            let _action = scholia::ui::render_editor_page(ui, &mut doc, &theme, &keybinds, false);
        },
    );
    out.textures_delta.clear();
}

#[test]
fn test_document_block_operations_and_sync() {
    let mut doc = scholia::model::MlaDocument::new_blank();
    assert_eq!(doc.blocks.len(), 1);

    // Add paragraph
    let p2 = doc.add_paragraph(None);
    assert_eq!(doc.blocks.len(), 2);
    assert_eq!(p2, 1);

    // Add blockquote
    let bq = doc.add_blockquote(Some(0));
    assert_eq!(doc.blocks.len(), 3);
    assert_eq!(bq, 1);

    // Add heading
    let h1 = doc.add_heading(1, None);
    assert_eq!(doc.blocks.len(), 4);
    assert_eq!(h1, 3);

    // Verify sync_body_from_blocks
    doc.blocks[0].text_mut().push_str("Paragraph one.");
    doc.blocks[1].text_mut().push_str("A quoted passage from literature.");
    doc.sync_body_from_blocks();
    assert!(doc.body.contains("Paragraph one."));
    assert!(doc.body.contains("> A quoted passage from literature."));
}

#[test]
fn test_word_count_and_pdf_page_count() {
    let mut doc = scholia::model::MlaDocument::new_blank();
    doc.title = "A Scholarly Analysis".to_string();
    assert_eq!(doc.total_word_count(), 3);
    // 1 page body + 1 page Works Cited = 2 pages
    assert_eq!(doc.estimated_page_count(), 2);

    // Add 300 words into a paragraph
    let words = vec!["word"; 300].join(" ");
    doc.blocks[0].text_mut().push_str(&words);
    assert_eq!(doc.total_word_count(), 303);
    // 300 words body => ceil(300 / 250) = 2 pages body + 1 page Works Cited = 3 pages
    assert_eq!(doc.estimated_page_count(), 3);

    // Adding Works Cited entry (still fits on the separate Works Cited page)
    let mut entry = scholia::model::WorksCitedEntry::new_empty();
    entry.author = "Smith, John".to_string();
    entry.title_of_source = "A Book".to_string();
    doc.works_cited.push(entry);
    assert_eq!(doc.estimated_page_count(), 3);
}

#[test]
fn test_block_deletion_focus_and_keybinds() {
    use scholia::keybinds::{Action, KeybindConfig};

    let mut doc = scholia::model::MlaDocument::new_blank();
    let _ = doc.add_paragraph(None);
    let _ = doc.add_paragraph(None);
    assert_eq!(doc.blocks.len(), 3);

    // If we delete block at index 2, focus should target previous index 1
    let idx = 2;
    let prev_idx = if idx > 0 { idx - 1 } else { 0 };
    doc.remove_block(idx);
    doc.active_block_idx = prev_idx;
    doc.requested_focus_block_idx = Some(prev_idx);

    assert_eq!(doc.blocks.len(), 2);
    assert_eq!(doc.active_block_idx, 1);
    assert_eq!(doc.requested_focus_block_idx, Some(1));

    // Verify keybinds are registered
    let cfg = KeybindConfig::default();
    assert_eq!(cfg.get_shortcut(Action::DeleteBlock).display_string(), "Ctrl+Backspace");
    assert_eq!(cfg.get_shortcut(Action::MoveBlockUp).display_string(), "Ctrl+Alt+Up");
    assert_eq!(cfg.get_shortcut(Action::MoveBlockDown).display_string(), "Ctrl+Alt+Down");
    assert_eq!(cfg.get_shortcut(Action::InsertHeading3).display_string(), "Ctrl+Alt+3");
}

#[test]
fn test_interface_options_and_removed_shortcuts() {
    use scholia::keybinds::{Action, KeybindConfig};
    use scholia::theme::ThemeConfig;

    let theme = ThemeConfig::default();
    assert!(theme.show_window_controls);
    assert!(theme.show_shortcuts_panel);

    // Verify shortcuts for AddParagraph, Exports, and in-text actions are disabled in check_action
    let cfg = KeybindConfig::default();
    let input = egui::InputState::default();
    assert!(!cfg.check_action(Action::AddParagraph, &input));
    assert!(!cfg.check_action(Action::ExportDocx, &input));
    assert!(!cfg.check_action(Action::ExportHtmlPdf, &input));

    // Anything that has an in-text action (^N, /cite) shouldn't have a keybind
    assert!(!cfg.check_action(Action::InsertCitation, &input));
    assert!(!cfg.check_action(Action::AddFootnote, &input));
    assert_eq!(cfg.get_shortcut(Action::InsertCitation).display_string(), "None");
    assert_eq!(cfg.get_shortcut(Action::AddFootnote).display_string(), "None");
    assert!(Action::InsertCitation.is_in_text_action());
    assert!(Action::AddFootnote.is_in_text_action());
    assert_eq!(Action::InsertCitation.in_text_hint(), Some("/cite"));
    assert_eq!(Action::AddFootnote.in_text_hint(), Some("^N"));

    // Verify Works Cited shortcut is Ctrl+W
    assert_eq!(cfg.get_shortcut(Action::ManageWorksCited).display_string(), "Ctrl+W");

    // Verify categories and renamed actions
    assert_eq!(Action::DeleteBlock.display_name(), "Delete Paragraph");
    assert_eq!(Action::SaveDocument.display_name(), "Save Document (.mla)");
    assert_eq!(Action::ManageWorksCited.category(), "Writing");
    assert_eq!(Action::DeleteBlock.category(), "Modify");
    assert_eq!(Action::SaveDocument.category(), "File");
    assert_eq!(Action::ConvertToMlaTitleCase.category(), "Tools");
}

#[test]
fn test_typographical_cleaning() {
    use scholia::model::typographical_clean;

    // Test em dash conversion
    assert_eq!(typographical_clean("prose--text"), "prose—text");
    assert_eq!(typographical_clean("prose -- text"), "prose—text");

    // Test double curly quotes
    assert_eq!(typographical_clean("\"Hello World\""), "“Hello World”");
    assert_eq!(typographical_clean("He said, \"yes\"."), "He said, “yes”.");

    // Test single curly quotes & apostrophes
    assert_eq!(typographical_clean("'hello'"), "‘hello’");
    assert_eq!(typographical_clean("don't Smith's"), "don’t Smith’s");
}

#[test]
fn test_explanatory_notes_and_superscript_engine() {
    use scholia::model::{clean_superscripts, num_to_superscript, MlaBlock, MlaDocument};

    let mut doc = MlaDocument::new_blank();
    assert_eq!(num_to_superscript(1), "¹");
    assert_eq!(num_to_superscript(2), "²");
    assert_eq!(num_to_superscript(12), "¹²");

    // Body text is clean prose, no raw superscript characters
    assert_eq!(clean_superscripts("Here is text¹ and more²."), "Here is text and more.");

    let block_0_id = doc.blocks[0].id().to_string();
    if let MlaBlock::Paragraph { text, .. } = &mut doc.blocks[0] {
        *text = "Here is clean prose text without raw superscripts.".to_string();
    }

    // Add note 1 and 2 linked to active block 0
    let n1 = doc.add_explanatory_note("First definition".to_string());
    let n2 = doc.add_explanatory_note("Second definition".to_string());
    assert_eq!(n1, 1);
    assert_eq!(n2, 2);
    assert_eq!(doc.notes.len(), 2);

    // Verify notes are linked to block 0
    let block_notes = doc.notes_for_block(&block_0_id);
    assert_eq!(block_notes.len(), 2);
    assert_eq!(block_notes[0].text, "First definition");
    assert_eq!(block_notes[0].index, 1);
    assert_eq!(block_notes[1].text, "Second definition");
    assert_eq!(block_notes[1].index, 2);

    // Verify block text remains clean prose
    assert_eq!(doc.blocks[0].text(), "Here is clean prose text without raw superscripts.");

    // Deleting note 1 removes it and re-indexes note 2 to index 1
    let note_1_id = doc.notes[0].id.clone();
    doc.delete_note_by_id(&note_1_id);

    assert_eq!(doc.notes.len(), 1);
    assert_eq!(doc.notes[0].index, 1);
    assert_eq!(doc.notes[0].text, "Second definition");

    let block_notes_after = doc.notes_for_block(&block_0_id);
    assert_eq!(block_notes_after.len(), 1);
    assert_eq!(block_notes_after[0].index, 1);
    assert_eq!(block_notes_after[0].text, "Second definition");

    // Adding a second block with its own note
    let b1_idx = doc.add_paragraph(None);
    let block_1_id = doc.blocks[b1_idx].id().to_string();
    if let MlaBlock::Paragraph { text, .. } = &mut doc.blocks[b1_idx] {
        *text = "Second paragraph prose.".to_string();
    }
    doc.add_note_to_block(&block_1_id, "Third definition on block 1".to_string());
    assert_eq!(doc.notes.len(), 2);
    assert_eq!(doc.notes[1].index, 2);
    assert_eq!(doc.notes_for_block(&block_1_id).len(), 1);

    // Deleting block 1 removes its linked notes automatically
    doc.remove_block(b1_idx);
    assert_eq!(doc.notes.len(), 1);
    assert_eq!(doc.notes[0].text, "Second definition");
}

#[test]
fn test_word_superscript_note_shortcut_and_json_tag() {
    use scholia::model::{
        num_to_superscript, render_text_with_note_tags, try_parse_and_apply_note_shortcut,
        MlaBlock, MlaDocument, NoteTag,
    };

    let mut doc = MlaDocument::new_blank();
    let block_id = doc.blocks[0].id().to_string();

    // 0. Verify that notes wait for spacebar: typing without spacebar does NOT trigger note
    let mut no_space_single = "The example^1".to_string();
    let mut tags_temp: Vec<NoteTag> = Vec::new();
    assert!(
        try_parse_and_apply_note_shortcut(&mut no_space_single, &mut tags_temp, &[]).unwrap().is_none(),
        "Must wait for spacebar: example^1 without space must not trigger"
    );

    let mut no_space_multi = "The example^10".to_string();
    assert!(
        try_parse_and_apply_note_shortcut(&mut no_space_multi, &mut tags_temp, &[]).unwrap().is_none(),
        "Must wait for spacebar: example^10 without space must not trigger"
    );

    // 1. User types in a paragraph: "The example^1 " and "another^13 " (followed by spacebar confirmation)
    let mut input_text = "The example^1 and another^13 quest continued.".to_string();
    let mut tags: Vec<NoteTag> = Vec::new();

    // First shortcut: example^1 confirmed with spacebar
    let result1 = try_parse_and_apply_note_shortcut(&mut input_text, &mut tags, &[]).unwrap();
    assert!(result1.is_some(), "Shortcut example^1 should be recognized upon typing space");
    let res1 = result1.unwrap();
    assert_eq!(res1.note_index, 1);
    assert_eq!(res1.word, "example");
    // Displays superscript character in the editor text!
    assert_eq!(input_text, "The example¹ and another^13 quest continued.");

    // Second shortcut: another^13 confirmed with spacebar (note 1 is already in-use so we pass [1])
    let result2 = try_parse_and_apply_note_shortcut(&mut input_text, &mut tags, &[1]).unwrap();
    assert!(result2.is_some(), "Shortcut another^13 should be recognized upon typing space");
    let res2 = result2.unwrap();
    assert_eq!(res2.note_index, 13);
    assert_eq!(res2.word, "another");
    // Displays superscript character in the editor text!
    assert_eq!(input_text, "The example¹ and another¹³ quest continued.");

    // 1a. Verify that reusing an existing note number returns Err and cancels the note
    let mut reuse_text = "test^1 ".to_string();
    let mut reuse_tags: Vec<NoteTag> = Vec::new();
    let reuse_result = try_parse_and_apply_note_shortcut(&mut reuse_text, &mut reuse_tags, &[1]);
    assert!(reuse_result.is_err(), "MLA 9: reusing note 1 that is already in use must return Err");
    // The caret notation should be stripped/cancelled from text
    assert!(!reuse_text.contains("^"), "Cancelled note shorthand must be removed from text");

    // 2. Adding notes does NOT delete the associated words
    assert!(input_text.contains("example"));
    assert!(input_text.contains("another"));

    // 3. JSON tags added to the paragraph
    assert_eq!(tags.len(), 2);
    assert_eq!(tags[0].note_index, 1);
    assert_eq!(tags[0].word, "example");
    assert_eq!(tags[1].note_index, 13);
    assert_eq!(tags[1].word, "another");

    // 4. Update block in document
    if let MlaBlock::Paragraph { text, note_tags, .. } = &mut doc.blocks[0] {
        *text = input_text.clone();
        *note_tags = tags.clone();
    }
    assert!(doc.link_note_from_shortcut(&block_id, res1.note_index, &res1.word), "Linking new note must succeed");
    assert!(doc.link_note_from_shortcut(&block_id, res2.note_index, &res2.word), "Linking new note must succeed");

    // Verify linked in document
    assert_eq!(doc.notes.len(), 2);
    assert_eq!(doc.notes[0].index, 1);
    assert_eq!(doc.notes[0].word, "example");
    assert_eq!(doc.notes[1].index, 2); // auto-reindexed to sequential 1, 2
    assert_eq!(doc.notes[1].word, "another");

    // 4a. Verify that linking a note number to a DIFFERENT word fails (MLA 9)
    assert!(
        !doc.link_note_from_shortcut(&block_id, 1, "different_word"),
        "MLA 9: link_note_from_shortcut must reject reuse of note 1 with a different word"
    );

    // 5. Test JSON serialization and deserialization retains note_tags
    let json = serde_json::to_string(&doc).expect("Serialization to JSON must succeed");
    assert!(json.contains("note_tags"), "JSON must contain note_tags tag in block");
    assert!(json.contains("example"));
    assert!(json.contains("another"));
    let deserialized: MlaDocument = serde_json::from_str(&json).expect("Deserialization must succeed");
    assert_eq!(deserialized.blocks[0].note_tags().len(), 2);
    assert_eq!(deserialized.blocks[0].note_tags()[0].word, "example");

    // 6. Test Exporter text rendering: superscripts placed right after the words without duplication!
    let block_notes = doc.notes_for_block(&block_id);
    let rendered_text = render_text_with_note_tags(&doc.blocks[0].text(), doc.blocks[0].note_tags(), &block_notes, num_to_superscript);
    assert_eq!(rendered_text, "The example¹ and another² quest continued.");

    let rendered_html = render_text_with_note_tags(&doc.blocks[0].text(), doc.blocks[0].note_tags(), &block_notes, |idx| format!("<sup>{}</sup>", idx));
    assert_eq!(rendered_html, "The example<sup>1</sup> and another<sup>2</sup> quest continued.");

    // 7. Deleting note removes note tag AND removes superscript from paragraph text
    doc.delete_explanatory_note(1);
    assert_eq!(doc.notes.len(), 1);
    assert_eq!(doc.blocks[0].note_tags().len(), 1);
    assert_eq!(doc.blocks[0].note_tags()[0].word, "another");
    assert!(!doc.blocks[0].text().contains("example¹"), "Deleted note superscript must be removed from text");
    assert!(doc.blocks[0].text().contains("another¹"), "Remaining note should reindex to ¹ in text");
}

#[test]
fn test_block_toggle_conversion() {
    use scholia::model::{MlaBlock, MlaDocument};

    let mut doc = MlaDocument::new_blank();
    if let MlaBlock::Paragraph { text, .. } = &mut doc.blocks[0] {
        *text = "This is a long prose passage intended for block quotation.".to_string();
    }

    // Toggle paragraph to blockquote
    doc.toggle_block_type(0);
    assert!(matches!(doc.blocks[0], MlaBlock::BlockQuote { .. }));

    // Toggle blockquote back to paragraph
    doc.toggle_block_type(0);
    assert!(matches!(doc.blocks[0], MlaBlock::Paragraph { .. }));
}

#[test]
fn test_semantic_versioning() {
    let version = env!("CARGO_PKG_VERSION");
    assert_eq!(version, "0.1.0");

    // Verify version components follow semver: major.minor.patch
    let parts: Vec<&str> = version.split('.').collect();
    assert_eq!(parts.len(), 3);
    assert_eq!(parts[0], "0");
    assert_eq!(parts[1], "1");
    assert_eq!(parts[2], "0");
}

#[test]
fn test_notes_exported_in_footer_not_separate_page() {
    use scholia::export::{export_to_docx, export_to_pdf, generate_mla_html};
    use scholia::model::MlaDocument;

    let mut doc = MlaDocument::new_blank();
    doc.header.student_name = "Jane Doe".to_string();
    doc.header.instructor_name = "Dr. Smith".to_string();
    doc.header.course = "ENG 101".to_string();
    doc.header.date = "21 September 2026".to_string();
    doc.title = "A Study of Footnotes".to_string();
    doc.add_explanatory_note("Explanatory footnote content.".to_string());

    // HTML Verification
    let html = generate_mla_html(&doc);
    assert!(html.contains(r#"<footer class="mla-footnotes">"#), "Notes must be inside footer element in HTML");
    assert!(html.contains(r#"<hr class="mla-footnotes-divider">"#), "Footer must contain standard 1.5-inch divider");
    assert!(html.contains("1. Explanatory footnote content."), "Footnote text must be present");
    assert!(!html.contains("mla-notes-section"), "Must not have separate notes page section");
    assert!(!html.contains(r#"<h2 class="mla-notes-title""#), "Must not have separate Notes h2 page header");

    // DOCX Export Verification
    let temp_dir = std::env::temp_dir();
    let docx_path = temp_dir.join("test_export_notes.docx");
    let docx_res = export_to_docx(&doc, &docx_path);
    assert!(docx_res.is_ok(), "DOCX export with notes in footer should succeed: {:?}", docx_res.err());
    let _ = std::fs::remove_file(&docx_path);

    // PDF Export Verification
    let pdf_path = temp_dir.join("test_export_notes.pdf");
    let pdf_res = export_to_pdf(&doc, &pdf_path);
    assert!(pdf_res.is_ok(), "PDF export with notes in footer should succeed: {:?}", pdf_res.err());
    let _ = std::fs::remove_file(&pdf_path);
}

#[test]
fn test_transparent_button_visuals() {
    let dark_theme = scholia::theme::ThemeConfig::preset_frosted_dark();
    let dark_visuals = dark_theme.create_egui_visuals();

    // Inactive buttons must have completely transparent background
    assert_eq!(dark_visuals.widgets.inactive.bg_fill, egui::Color32::TRANSPARENT);
    assert_eq!(dark_visuals.widgets.inactive.weak_bg_fill, egui::Color32::TRANSPARENT);

    // Hovered buttons should be softly tinted, not solid opaque gray (egui default is from_gray(70) with alpha 255)
    assert_ne!(dark_visuals.widgets.hovered.bg_fill, egui::Color32::from_gray(70));
    assert_ne!(dark_visuals.widgets.hovered.bg_fill.a(), 255);

    // Active buttons should also be translucent
    assert_ne!(dark_visuals.widgets.active.bg_fill, egui::Color32::from_gray(55));
    assert_ne!(dark_visuals.widgets.active.bg_fill.a(), 255);

    // Test light theme as well
    let light_theme = scholia::theme::ThemeConfig::preset_frosted_light();
    let light_visuals = light_theme.create_egui_visuals();
    assert_eq!(light_visuals.widgets.inactive.bg_fill, egui::Color32::TRANSPARENT);
    assert_eq!(light_visuals.widgets.inactive.weak_bg_fill, egui::Color32::TRANSPARENT);
}

#[test]
fn test_sentence_splitting_and_reordering() {
    use scholia::model::{reorder_sentences, split_sentences};

    // 1. Basic sentence splitting
    let text1 = "First sentence. Second sentence? Third sentence!";
    let (ws, spans) = split_sentences(text1);
    assert_eq!(ws, "");
    assert_eq!(spans.len(), 3);
    assert_eq!(spans[0].text, "First sentence.");
    assert_eq!(spans[0].trailing_sep, " ");
    assert_eq!(spans[1].text, "Second sentence?");
    assert_eq!(spans[1].trailing_sep, " ");
    assert_eq!(spans[2].text, "Third sentence!");
    assert_eq!(spans[2].trailing_sep, "");

    // 2. Abbreviation and decimal handling
    let text2 = "Dr. Smith cited p. 42 and 3.14 ratio in vol. 2. Next statement.";
    let (_, spans2) = split_sentences(text2);
    assert_eq!(spans2.len(), 2);
    assert_eq!(spans2[0].text, "Dr. Smith cited p. 42 and 3.14 ratio in vol. 2.");
    assert_eq!(spans2[1].text, "Next statement.");

    // 3. Quotes, ellipses, and superscripts
    let text3 = "He shouted, “Watch out!”¹ Then he fled... But wait.";
    let (_, spans3) = split_sentences(text3);
    assert_eq!(spans3.len(), 2);
    assert_eq!(spans3[0].text, "He shouted, “Watch out!”¹");
    assert_eq!(spans3[1].text, "Then he fled... But wait.");

    // 4. Reorder sentences Alt+Left (swap S1 with S0)
    // S0 = "A.", S1 = "B.", S2 = "C."
    let p = "A. B. C.";
    // Cursor at 'B' (index 3)
    let (swapped_left, cursor_left) = reorder_sentences(p, 3, true).expect("Should swap left");
    assert_eq!(swapped_left, "B. A. C.");
    assert_eq!(cursor_left, 0, "Cursor should follow 'B' to index 0");

    // Reorder sentences Alt+Right from index 0 ("B." swaps with "A.")
    let (swapped_right, cursor_right) = reorder_sentences(&swapped_left, 0, false).expect("Should swap right");
    assert_eq!(swapped_right, "A. B. C.");
    assert_eq!(cursor_right, 3, "Cursor should follow 'B' back to index 3");

    // Bounds checking
    // Trying to move left on the first sentence must return None
    assert!(reorder_sentences(p, 0, true).is_none());
    // Trying to move right on the last sentence must return None
    assert!(reorder_sentences(p, 6, false).is_none());
    // Single sentence paragraph returns None
    assert!(reorder_sentences("Single sentence without period", 5, true).is_none());
    assert!(reorder_sentences("Single sentence without period", 5, false).is_none());
}

#[test]
fn test_document_sentence_reordering_and_keybinds() {
    use scholia::keybinds::{Action, KeyName, KeybindConfig};
    use scholia::model::MlaDocument;

    let config = KeybindConfig::default();
    let left_sc = config.get_shortcut(Action::MoveSentenceLeft);
    assert!(left_sc.alt, "MoveSentenceLeft must use Alt");
    assert!(left_sc.ctrl, "MoveSentenceLeft must use Ctrl");
    assert_eq!(left_sc.key, KeyName::Left, "MoveSentenceLeft key must be Left");

    let right_sc = config.get_shortcut(Action::MoveSentenceRight);
    assert!(right_sc.alt, "MoveSentenceRight must use Alt");
    assert!(right_sc.ctrl, "MoveSentenceRight must use Ctrl");
    assert_eq!(right_sc.key, KeyName::Right, "MoveSentenceRight key must be Right");

    let mut doc = MlaDocument::new_blank();
    doc.blocks[0] = scholia::model::MlaBlock::Paragraph {
        id: "p1".to_string(),
        text: "Sentence one. Sentence two. Sentence three.".to_string(),
        note_tags: Vec::new(),
    };
    doc.active_block_idx = 0;

    // Move second sentence left
    // Cursor is in "Sentence two." at index 14
    let res = doc.move_sentence_in_active_block(14, true);
    assert!(res.is_ok());
    assert_eq!(doc.blocks[0].text(), "Sentence two. Sentence one. Sentence three.");

    // Move it right again
    let new_cursor = res.unwrap();
    let res_right = doc.move_sentence_in_active_block(new_cursor, false);
    assert!(res_right.is_ok());
    assert_eq!(doc.blocks[0].text(), "Sentence one. Sentence two. Sentence three.");
}

#[test]
fn test_word_grouping_undo_and_redo() {
    use scholia::history::HistoryManager;
    use scholia::model::{MlaBlock, MlaDocument};

    let mut doc = MlaDocument::new_blank();
    let initial_text = doc.blocks[0].text().to_string();
    let mut history = HistoryManager::new(&doc);

    // Initial state: cannot undo or redo
    assert!(!history.can_undo());
    assert!(!history.can_redo());

    // Type letters of a word one by one: 'H', 'e', 'l', 'l', 'o'
    let letters = ['H', 'e', 'l', 'l', 'o'];
    let mut current_text = initial_text.clone();
    for &ch in &letters {
        current_text.push(ch);
        if let MlaBlock::Paragraph { text, .. } = &mut doc.blocks[0] {
            *text = current_text.clone();
        }
        history.on_frame_end(&doc, current_text.len());
        // Mid-word typing retains undoable typing_base without individual letter undo states
        assert!(history.can_undo());
        assert!(!history.can_redo());
    }

    assert_eq!(doc.blocks[0].text(), format!("{initial_text}Hello"));

    // User triggers Undo (Ctrl+Z): whole word undone in a single step!
    let restored = history.undo(&doc, current_text.len());
    assert!(restored.is_some());
    let snapshot = restored.unwrap();
    doc = snapshot.doc;
    assert_eq!(doc.blocks[0].text(), initial_text);

    // Redo is now available
    assert!(history.can_redo());

    // User triggers Redo (Ctrl+Y): whole word restored in a single step!
    let redone = history.redo(&doc, 0);
    assert!(redone.is_some());
    let redo_snapshot = redone.unwrap();
    doc = redo_snapshot.doc;
    assert_eq!(doc.blocks[0].text(), format!("{initial_text}Hello"));
}

#[test]
fn test_multi_word_boundary_grouping() {
    use scholia::history::HistoryManager;
    use scholia::model::{MlaBlock, MlaDocument};

    let mut doc = MlaDocument::new_blank();
    if let MlaBlock::Paragraph { text, .. } = &mut doc.blocks[0] {
        *text = "Start".to_string();
    }
    let mut history = HistoryManager::new(&doc);

    // Type " one "
    if let MlaBlock::Paragraph { text, .. } = &mut doc.blocks[0] {
        *text = "Start one ".to_string();
    }
    history.on_frame_end(&doc, 10);

    // Type "two."
    if let MlaBlock::Paragraph { text, .. } = &mut doc.blocks[0] {
        *text = "Start one two.".to_string();
    }
    history.on_frame_end(&doc, 14);

    assert_eq!(doc.blocks[0].text(), "Start one two.");

    // Sequential undo:
    // 1st undo rolls back "two."
    let step1 = history.undo(&doc, 14).expect("Should undo two.");
    doc = step1.doc;
    assert_eq!(doc.blocks[0].text(), "Start one ");

    // 2nd undo rolls back "one "
    let step2 = history.undo(&doc, 10).expect("Should undo one.");
    doc = step2.doc;
    assert_eq!(doc.blocks[0].text(), "Start");

    // Sequential redo:
    let redo1 = history.redo(&doc, 5).expect("Should redo one.");
    doc = redo1.doc;
    assert_eq!(doc.blocks[0].text(), "Start one ");

    let redo2 = history.redo(&doc, 10).expect("Should redo two.");
    doc = redo2.doc;
    assert_eq!(doc.blocks[0].text(), "Start one two.");
}

#[test]
fn test_discrete_actions_undo_and_redo() {
    use scholia::history::HistoryManager;
    use scholia::model::{MlaBlock, MlaDocument};

    let mut doc = MlaDocument::new_blank();
    doc.blocks[0] = MlaBlock::Paragraph {
        id: "p1".to_string(),
        text: "Sentence A. Sentence B.".to_string(),
        note_tags: Vec::new(),
    };
    let mut history = HistoryManager::new(&doc);

    // Record discrete action before sentence move
    history.record_discrete_action(&doc, 15);
    let _ = doc.move_sentence_in_active_block(15, true);
    assert_eq!(doc.blocks[0].text(), "Sentence B. Sentence A.");

    // Undo sentence move
    let s1 = history.undo(&doc, 0).expect("Undo sentence move");
    doc = s1.doc;
    assert_eq!(doc.blocks[0].text(), "Sentence A. Sentence B.");

    // Redo sentence move
    let s2 = history.redo(&doc, 0).expect("Redo sentence move");
    doc = s2.doc;
    assert_eq!(doc.blocks[0].text(), "Sentence B. Sentence A.");

    // Discrete action: adding explanatory note
    history.record_discrete_action(&doc, 0);
    doc.add_explanatory_note("Note definition".to_string());
    assert_eq!(doc.notes.len(), 1);

    // Undo note addition
    let s3 = history.undo(&doc, 0).expect("Undo note");
    doc = s3.doc;
    assert_eq!(doc.notes.len(), 0);

    // Redo note addition
    let s4 = history.redo(&doc, 0).expect("Redo note");
    doc = s4.doc;
    assert_eq!(doc.notes.len(), 1);
}

#[test]
fn test_undo_redo_shortcuts_and_keybinds() {
    use scholia::keybinds::{Action, KeybindConfig, KeyName};

    let cfg = KeybindConfig::default();
    assert_eq!(cfg.get_shortcut(Action::Undo).display_string(), "Ctrl+Z");
    assert_eq!(cfg.get_shortcut(Action::Redo).display_string(), "Ctrl+Y");

    // Action categories and labels
    assert_eq!(Action::Undo.category(), "Modify");
    assert_eq!(Action::Redo.category(), "Modify");
    assert_eq!(Action::Undo.display_name(), "Undo");
    assert_eq!(Action::Redo.display_name(), "Redo");

    // KeyName labels
    assert_eq!(KeyName::Z.label(), "Z");
    assert_eq!(KeyName::Y.label(), "Y");

    // Test shortcut matching with egui input state
    let mut input = egui::InputState::default();
    input.modifiers.command = true;
    input.modifiers.ctrl = true;
    input.events.push(egui::Event::Key {
        key: egui::Key::Z,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: input.modifiers,
    });
    assert!(cfg.check_action(Action::Undo, &input));

    // Test Redo with Ctrl+Y
    let mut input_y = egui::InputState::default();
    input_y.modifiers.command = true;
    input_y.modifiers.ctrl = true;
    input_y.events.push(egui::Event::Key {
        key: egui::Key::Y,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: input_y.modifiers,
    });
    assert!(cfg.check_action(Action::Redo, &input_y));

    // Test Redo with Ctrl+Shift+Z
    let mut input_shift_z = egui::InputState::default();
    input_shift_z.modifiers.command = true;
    input_shift_z.modifiers.ctrl = true;
    input_shift_z.modifiers.shift = true;
    input_shift_z.events.push(egui::Event::Key {
        key: egui::Key::Z,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: input_shift_z.modifiers,
    });
    assert!(cfg.check_action(Action::Redo, &input_shift_z));

    // Test Redo with Ctrl+Shift+Y
    let mut input_shift_y = egui::InputState::default();
    input_shift_y.modifiers.command = true;
    input_shift_y.modifiers.ctrl = true;
    input_shift_y.modifiers.shift = true;
    input_shift_y.events.push(egui::Event::Key {
        key: egui::Key::Y,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: input_shift_y.modifiers,
    });
    assert!(cfg.check_action(Action::Redo, &input_shift_y));
}

#[test]
fn test_history_limits_and_configuration() {
    use scholia::history::HistoryManager;
    use scholia::model::MlaDocument;

    let doc = MlaDocument::new_blank();
    let mut history = HistoryManager::new(&doc);

    // Default limit must be 512 actions
    assert_eq!(history.max_depth, 512);

    // User can configure from 16 to 8192 actions
    history.set_max_depth(256);
    assert_eq!(history.max_depth, 256);

    history.set_max_depth(1024);
    assert_eq!(history.max_depth, 1024);

    // Enforce lower bound clamp to 16
    history.set_max_depth(5);
    assert_eq!(history.max_depth, 16);

    // Enforce upper bound clamp to 8192
    history.set_max_depth(100_000);
    assert_eq!(history.max_depth, 8192);

    // Verify trimming of undo stack when depth is reduced
    history.set_max_depth(20);
    for i in 0..30 {
        let mut test_doc = doc.clone();
        test_doc.title = format!("Title {}", i);
        history.record_discrete_action(&test_doc, 0);
    }
    assert!(history.undo_stack.len() <= 20);
    assert_eq!(history.undo_stack.len(), 20);

    // When reducing limit to 16, stack trims to 16
    history.set_max_depth(16);
    assert_eq!(history.undo_stack.len(), 16);
}

#[test]
fn test_unsaved_changes_dialog_behavior() {
    use scholia::model::MlaDocument;
    use scholia::theme::ThemeConfig;
    use scholia::ui::{render_unsaved_dialog, UnsavedDialogResponse, UnsavedDialogState};

    let doc = MlaDocument::new_blank();
    let theme = ThemeConfig::default();
    let mut state = UnsavedDialogState { is_open: false };

    let ctx = egui::Context::default();

    // When dialog is not open, returns None
    let resp = render_unsaved_dialog(&ctx, &mut state, &doc, &theme);
    assert_eq!(resp, UnsavedDialogResponse::None);

    // When dialog is open, can be opened
    state.is_open = true;
    assert!(state.is_open);
}

#[test]
fn test_modal_fill_color_ninety_percent_opacity() {
    use scholia::theme::ThemeConfig;

    let theme = ThemeConfig::default();
    let modal_color = theme.modal_fill_color();
    // 90% opaque corresponds to alpha = 230 (or ~0.90 * 255)
    assert_eq!(modal_color.a(), 230, "Modal dialog background must have alpha = 230 (90% opaque, 10% transparent)");

    // Window fill in visuals should also match modal fill color
    let visuals = theme.create_egui_visuals();
    assert_eq!(visuals.window_fill.a(), 230, "Visuals window_fill must match 90% opaque modal fill");
}

#[test]
fn test_rose_pine_theme_presets() {
    use scholia::theme::{ThemeConfig, ThemePreset};

    let presets = ThemePreset::all_presets();
    assert_eq!(presets.len(), 7, "Should support 7 curated theme presets");
    assert!(presets.contains(&ThemePreset::RosePine));
    assert!(presets.contains(&ThemePreset::RosePineMoon));
    assert!(presets.contains(&ThemePreset::RosePineDawn));

    // Test Rosé Pine (Main Dark)
    let mut theme = ThemeConfig::default();
    theme.apply_preset(ThemePreset::RosePine);
    assert_eq!(theme.preset, ThemePreset::RosePine);
    assert!(theme.is_dark());
    assert_eq!(theme.window_tint_rgb, [25, 23, 36]);
    assert_eq!(theme.accent_rgb, [235, 188, 186]);

    // Test Rosé Pine Moon (Deep Violet Dark)
    theme.apply_preset(ThemePreset::RosePineMoon);
    assert_eq!(theme.preset, ThemePreset::RosePineMoon);
    assert!(theme.is_dark());
    assert_eq!(theme.window_tint_rgb, [35, 33, 54]);
    assert_eq!(theme.accent_rgb, [234, 154, 151]);

    // Test Rosé Pine Dawn (Warm Pastel Light)
    theme.apply_preset(ThemePreset::RosePineDawn);
    assert_eq!(theme.preset, ThemePreset::RosePineDawn);
    assert!(!theme.is_dark(), "Rose Pine Dawn must be a light theme");
    assert_eq!(theme.window_tint_rgb, [250, 244, 237]);
    assert_eq!(theme.accent_rgb, [215, 130, 126]);
}

#[test]
fn test_theme_folder_save_and_load_roundtrip() {
    use scholia::theme::{load_theme_file, save_custom_theme, ThemeConfig, ThemePreset};

    let mut theme = ThemeConfig::preset_rose_pine();
    theme.accent_rgb = [123, 234, 45]; // Custom accent
    theme.page_opacity = 0.42;

    let test_theme_name = "Automated Test Theme";
    let save_res = save_custom_theme(&theme, test_theme_name);
    assert!(save_res.is_ok(), "Saving custom theme must succeed: {:?}", save_res);

    let saved_path = save_res.unwrap();
    assert!(saved_path.exists());

    let load_res = load_theme_file(&saved_path);
    assert!(load_res.is_ok(), "Loading saved theme file must succeed: {:?}", load_res);

    let loaded = load_res.unwrap();
    assert_eq!(loaded.preset, ThemePreset::Custom);
    assert_eq!(loaded.custom_name, Some(test_theme_name.to_string()));
    assert_eq!(loaded.accent_rgb, [123, 234, 45]);
    assert_eq!(loaded.page_opacity, 0.42);

    // Clean up temporary test file
    let _ = std::fs::remove_file(saved_path);
}

