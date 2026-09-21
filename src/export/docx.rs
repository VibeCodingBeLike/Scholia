use crate::model::{MlaBlock, MlaDocument};
use docx_rs::{
    AlignmentType, Docx, Footer, Header, LineSpacing, LineSpacingType, PageMargin, PageNum, Paragraph, Run,
    RunFonts, SpecialIndentType,
};
use std::fs::File;
use std::path::Path;

fn double_spacing() -> LineSpacing {
    LineSpacing::new()
        .line(480)
        .line_rule(LineSpacingType::Auto)
}

pub fn export_to_docx(doc: &MlaDocument, path: &Path) -> Result<(), String> {
    let font_name = "Times New Roman";
    let font_size = 24; // 24 half-points = 12 pt

    let mut docx = Docx::new();

    // 1-inch margins all around (1440 dxa = 1 in), 0.5 inch header margin (720 dxa)
    docx = docx.page_margin(
        PageMargin::new()
            .top(1440)
            .bottom(1440)
            .left(1440)
            .right(1440)
            .header(720)
            .footer(720),
    );

    // Running Header: [LastName] [PageNum], aligned right
    let last_name = doc.header.derived_last_name();
    let header = Header::new().add_paragraph(
        Paragraph::new()
            .align(AlignmentType::Right)
            .line_spacing(double_spacing())
            .add_run(
                Run::new()
                    .fonts(RunFonts::new().ascii(font_name))
                    .size(font_size)
                    .add_text(format!("{} ", last_name)),
            )
            .add_page_num(PageNum::new()),
    );
    docx = docx.header(header);

    // --- Page 1 Heading ---
    // Student Name
    docx = docx.add_paragraph(
        Paragraph::new()
            .align(AlignmentType::Left)
            .line_spacing(double_spacing())
            .add_run(
                Run::new()
                    .fonts(RunFonts::new().ascii(font_name))
                    .size(font_size)
                    .add_text(&doc.header.student_name),
            ),
    );
    // Instructor Name
    docx = docx.add_paragraph(
        Paragraph::new()
            .align(AlignmentType::Left)
            .line_spacing(double_spacing())
            .add_run(
                Run::new()
                    .fonts(RunFonts::new().ascii(font_name))
                    .size(font_size)
                    .add_text(&doc.header.instructor_name),
            ),
    );
    // Course
    docx = docx.add_paragraph(
        Paragraph::new()
            .align(AlignmentType::Left)
            .line_spacing(double_spacing())
            .add_run(
                Run::new()
                    .fonts(RunFonts::new().ascii(font_name))
                    .size(font_size)
                    .add_text(&doc.header.course),
            ),
    );
    // Date
    docx = docx.add_paragraph(
        Paragraph::new()
            .align(AlignmentType::Left)
            .line_spacing(double_spacing())
            .add_run(
                Run::new()
                    .fonts(RunFonts::new().ascii(font_name))
                    .size(font_size)
                    .add_text(&doc.header.date),
            ),
    );

    // --- Paper Title (Centered, Standard 12pt, Not bold) ---
    docx = docx.add_paragraph(
        Paragraph::new()
            .align(AlignmentType::Center)
            .line_spacing(double_spacing())
            .add_run(
                Run::new()
                    .fonts(RunFonts::new().ascii(font_name))
                    .size(font_size)
                    .add_text(&doc.title),
            ),
    );

    // --- Body Blocks ---
    for block in &doc.blocks {
        let block_notes = doc.notes_for_block(block.id());

        match block {
            MlaBlock::Paragraph { text, .. } => {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    let formatted_text = crate::model::render_text_with_note_tags(
                        trimmed,
                        block.note_tags(),
                        &block_notes,
                        crate::model::num_to_superscript,
                    );
                    // MLA First-Line Indent: 0.5 in (720 dxa)
                    let p = Paragraph::new()
                        .align(AlignmentType::Left)
                        .line_spacing(double_spacing())
                        .indent(None, Some(SpecialIndentType::FirstLine(720)), None, None);

                    let p = append_formatted_runs(p, &formatted_text, font_name, font_size);
                    docx = docx.add_paragraph(p);
                }
            }
            MlaBlock::BlockQuote { text, citation, .. } => {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    let formatted_text = crate::model::render_text_with_note_tags(
                        trimmed,
                        block.note_tags(),
                        &block_notes,
                        crate::model::num_to_superscript,
                    );
                    // MLA Blockquote: 0.5 in left margin, double spaced, citation outside
                    let mut p = Paragraph::new()
                        .align(AlignmentType::Left)
                        .line_spacing(double_spacing())
                        .indent(Some(720), None, None, None);

                    p = append_formatted_runs(p, &formatted_text, font_name, font_size);
                    if !citation.trim().is_empty() {
                        p = p.add_run(
                            Run::new()
                                .fonts(RunFonts::new().ascii(font_name))
                                .size(font_size)
                                .add_text(format!(" {}", citation.trim())),
                        );
                    }
                    docx = docx.add_paragraph(p);
                }
            }
            MlaBlock::SectionHeading { level, text, .. } => {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    let mut run = Run::new()
                        .fonts(RunFonts::new().ascii(font_name))
                        .size(font_size)
                        .add_text(trimmed);

                    let mut p = Paragraph::new().line_spacing(double_spacing());

                    match level {
                        1 => {
                            // Level 1: Bold, flush left
                            run = run.bold();
                            p = p.align(AlignmentType::Left);
                        }
                        2 => {
                            // Level 2: Italics, flush left
                            run = run.italic();
                            p = p.align(AlignmentType::Left);
                        }
                        _ => {
                            // Level 3: Bold, centered
                            run = run.bold();
                            p = p.align(AlignmentType::Center);
                        }
                    }

                    p = p.add_run(run);
                    docx = docx.add_paragraph(p);
                }
            }
        }
    }

    // Explanatory Notes / Footnotes (Rendered in page footer per MLA standards)
    if !doc.notes.is_empty() {
        let mut footer = Footer::new();
        // 1.5-inch rule divider
        footer = footer.add_paragraph(
            Paragraph::new()
                .align(AlignmentType::Left)
                .add_run(
                    Run::new()
                        .fonts(RunFonts::new().ascii(font_name))
                        .size(20) // 10pt
                        .add_text("______________________"),
                ),
        );

        for note in &doc.notes {
            let note_text = format!("{}. {}", note.index, crate::model::typographical_clean(&note.text));
            footer = footer.add_paragraph(
                Paragraph::new()
                    .align(AlignmentType::Left)
                    .indent(None, Some(SpecialIndentType::FirstLine(720)), None, None)
                    .add_run(
                        Run::new()
                            .fonts(RunFonts::new().ascii(font_name))
                            .size(20) // 10pt
                            .add_text(note_text),
                    ),
            );
        }
        docx = docx.footer(footer);
    }

    // --- Works Cited (Starts on a New Page) ---
    let wc_title = if doc.works_cited.len() == 1 {
        "Work Cited"
    } else {
        "Works Cited"
    };

    docx = docx.add_paragraph(
        Paragraph::new()
            .page_break_before(true)
            .align(AlignmentType::Center)
            .line_spacing(double_spacing())
            .add_run(
                Run::new()
                    .fonts(RunFonts::new().ascii(font_name))
                    .size(font_size)
                    .add_text(wc_title),
            ),
    );

    if doc.works_cited.is_empty() {
        docx = docx.add_paragraph(
            Paragraph::new()
                .align(AlignmentType::Center)
                .line_spacing(double_spacing())
                .add_run(
                    Run::new()
                        .fonts(RunFonts::new().ascii(font_name))
                        .size(font_size)
                        .add_text("No entries yet."),
                ),
        );
    } else {
        // Sorted works cited entries with hanging indent (left 720, hanging 720)
        let mut sorted = doc.works_cited.clone();
        sorted.sort_by_key(|a| a.sort_key());

        for entry in &sorted {
            let markdown = entry.format_markdown();
            let p = Paragraph::new()
                .align(AlignmentType::Left)
                .line_spacing(double_spacing())
                .indent(Some(720), Some(SpecialIndentType::Hanging(720)), None, None);

            let p = append_formatted_runs(p, &markdown, font_name, font_size);
            docx = docx.add_paragraph(p);
        }
    }

    let file = File::create(path).map_err(|e| format!("Failed to create file: {}", e))?;
    docx.build()
        .pack(file)
        .map_err(|e| format!("Failed to package docx: {:?}", e))?;

    Ok(())
}

/// Parses inline markdown asterisks for italics: *title* into Runs, with typographical cleaning
fn append_formatted_runs(
    mut paragraph: Paragraph,
    text: &str,
    font_name: &str,
    font_size: usize,
) -> Paragraph {
    let parts: Vec<&str> = text.split('*').collect();
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        let is_italic = i % 2 == 1; // Odd indices are inside *...*
        let cleaned = crate::model::typographical_clean(part);
        let mut run = Run::new()
            .fonts(RunFonts::new().ascii(font_name))
            .size(font_size)
            .add_text(cleaned);
        if is_italic {
            run = run.italic();
        }
        paragraph = paragraph.add_run(run);
    }
    paragraph
}
