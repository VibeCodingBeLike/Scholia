use scholia::export::{export_to_docx, export_to_html, export_to_text};
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
    let doc = MlaDocument::sample_template();
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
    assert_eq!(doc.estimated_page_count(), 1);

    // Add 300 words into a paragraph
    let words = vec!["word"; 300].join(" ");
    doc.blocks[0].text_mut().push_str(&words);
    assert_eq!(doc.total_word_count(), 303);
    // 300 words body => ceil(300 / 250) = 2 pages
    assert_eq!(doc.estimated_page_count(), 2);

    // Adding Works Cited entry adds 1 page for the separate Works Cited page
    let mut entry = scholia::model::WorksCitedEntry::new_empty();
    entry.author = "Smith, John".to_string();
    entry.title_of_source = "A Book".to_string();
    doc.works_cited.push(entry);
    assert_eq!(doc.estimated_page_count(), 3);
}
