use crate::fonts::EMBEDDED_TIMES_NEW_ROMAN;
use crate::model::{MlaBlock, MlaDocument};
use printpdf::{
    FontId, Mm, Op, ParsedFont, PdfDocument, PdfFontHandle, PdfFontParseWarning, PdfPage,
    PdfSaveOptions, PdfWarnMsg, Point, Pt, TextItem as PdfTextItem,
};
use std::fs;
use std::path::Path;

// ── MLA 9 layout constants ───────────────────────────────────────────────────
const PAGE_W_MM: f32 = 215.9; // US Letter width
const PAGE_H_MM: f32 = 279.4; // US Letter height
const MARGIN_MM: f32 = 25.4; // 1-inch margins
const FONT_PT: f32 = 12.0;
const LINE_H_PT: f32 = FONT_PT * 2.0; // MLA double-spacing
const INDENT_MM: f32 = 12.7; // 0.5-inch paragraph first-line indent
const BLOCKQUOTE_INDENT_MM: f32 = 12.7; // 0.5-inch block quote left indent

fn pt_per_mm() -> f32 { 2.834645669 }
fn text_w_mm() -> f32 { PAGE_W_MM - 2.0 * MARGIN_MM }
fn top_y_mm() -> f32 { PAGE_H_MM - MARGIN_MM }
fn running_head_y_mm() -> f32 { PAGE_H_MM - MARGIN_MM / 2.0 }
fn line_h_mm() -> f32 { LINE_H_PT / pt_per_mm() }

/// Rough average character width in mm for Times New Roman 12pt
fn char_w_mm() -> f32 { FONT_PT * 0.45 / pt_per_mm() }

pub fn export_to_pdf(doc: &MlaDocument, path: &Path) -> Result<(), String> {
    let bytes = build_pdf(doc)?;
    fs::write(path, bytes).map_err(|e| format!("Failed to write PDF: {e}"))?;
    Ok(())
}

// ── Logical line model ───────────────────────────────────────────────────────

#[derive(Debug, Clone)]
enum LineKind {
    Normal,
    Bold,
    Centered,
    PageBreak,
}

#[derive(Debug, Clone)]
struct DocLine {
    kind: LineKind,
    text: String,
    /// extra left indent in mm beyond the base margin
    left_extra_mm: f32,
    /// first-line additional indent in mm
    first_indent_mm: f32,
}

impl DocLine {
    fn normal(text: impl Into<String>) -> Self {
        Self { kind: LineKind::Normal, text: text.into(), left_extra_mm: 0.0, first_indent_mm: 0.0 }
    }
    fn bold(text: impl Into<String>) -> Self {
        Self { kind: LineKind::Bold, text: text.into(), left_extra_mm: 0.0, first_indent_mm: 0.0 }
    }
    fn centered(text: impl Into<String>) -> Self {
        Self { kind: LineKind::Centered, text: text.into(), left_extra_mm: 0.0, first_indent_mm: 0.0 }
    }
    fn paragraph(text: impl Into<String>) -> Self {
        Self { kind: LineKind::Normal, text: text.into(), left_extra_mm: 0.0, first_indent_mm: INDENT_MM }
    }
    fn blockquote(text: impl Into<String>) -> Self {
        Self { kind: LineKind::Normal, text: text.into(), left_extra_mm: BLOCKQUOTE_INDENT_MM, first_indent_mm: 0.0 }
    }
    fn page_break() -> Self {
        Self { kind: LineKind::PageBreak, text: String::new(), left_extra_mm: 0.0, first_indent_mm: 0.0 }
    }
}

// ── Main PDF builder ─────────────────────────────────────────────────────────

fn build_pdf(mla: &MlaDocument) -> Result<Vec<u8>, String> {
    let mut warn: Vec<PdfWarnMsg> = Vec::new();
    let mut pdf = PdfDocument::new(&mla.title);

    let mut font_warn: Vec<PdfFontParseWarning> = Vec::new();
    let parsed = ParsedFont::from_bytes(EMBEDDED_TIMES_NEW_ROMAN, 0, &mut font_warn)
        .ok_or("Failed to parse bundled Times New Roman")?;
    let font = pdf.add_font(&parsed);

    let last_name = mla.header.derived_last_name();
    let mut sorted_wc = mla.works_cited.clone();
    sorted_wc.sort_by_key(|a| a.sort_key());

    // ── Build logical document lines ─────────────────────────────────────────
    let doc_lines = rebuild_lines(mla, &sorted_wc);

    // ── Render pages ─────────────────────────────────────────────────────────
    let mut pages: Vec<PdfPage> = Vec::new();
    let mut page_ops: Vec<Op> = Vec::new();
    let mut y_mm = top_y_mm();
    let mut page_num: u32 = 1;
    let mut added_notes_footer = false;

    // Draw running head (right-aligned "LastName N") at top of each page
    add_running_head(&mut page_ops, &font, &last_name, page_num);

    for line in &doc_lines {
        if matches!(line.kind, LineKind::PageBreak) {
            if !added_notes_footer && !mla.notes.is_empty() {
                add_footnotes_footer(&mut page_ops, &font, &mla.notes);
                added_notes_footer = true;
            }
            pages.push(PdfPage::new(Mm(PAGE_W_MM), Mm(PAGE_H_MM), page_ops));
            page_ops = Vec::new();
            y_mm = top_y_mm();
            page_num += 1;
            add_running_head(&mut page_ops, &font, &last_name, page_num);
            continue;
        }

        // Page overflow check
        if y_mm - line_h_mm() < MARGIN_MM {
            pages.push(PdfPage::new(Mm(PAGE_W_MM), Mm(PAGE_H_MM), page_ops));
            page_ops = Vec::new();
            y_mm = top_y_mm();
            page_num += 1;
            add_running_head(&mut page_ops, &font, &last_name, page_num);
        }

        let x_mm = if matches!(line.kind, LineKind::Centered) {
            // Center: compute approximate text width and offset
            let approx_w = line.text.len() as f32 * char_w_mm();
            MARGIN_MM + (text_w_mm() - approx_w) / 2.0
        } else {
            MARGIN_MM + line.left_extra_mm + line.first_indent_mm
        };

        // Write the text line
        page_ops.push(Op::StartTextSection);
        page_ops.push(Op::SetTextCursor { pos: Point::new(Mm(x_mm), Mm(y_mm)) });
        page_ops.push(Op::SetFont {
            font: PdfFontHandle::External(font.clone()),
            size: Pt(FONT_PT),
        });
        page_ops.push(Op::SetLineHeight { lh: Pt(LINE_H_PT) });

        if matches!(line.kind, LineKind::Bold) {
            page_ops.push(Op::SetTextRenderingMode {
                mode: printpdf::TextRenderingMode::FillStroke,
            });
            page_ops.push(Op::SetOutlineThickness { pt: Pt(0.25) });
        }

        page_ops.push(Op::ShowText {
            items: vec![PdfTextItem::Text(line.text.clone())],
        });

        if matches!(line.kind, LineKind::Bold) {
            page_ops.push(Op::SetTextRenderingMode {
                mode: printpdf::TextRenderingMode::Fill,
            });
        }

        page_ops.push(Op::EndTextSection);

        y_mm -= line_h_mm();
    }

    if !added_notes_footer && !mla.notes.is_empty() {
        add_footnotes_footer(&mut page_ops, &font, &mla.notes);
    }

    if !page_ops.is_empty() {
        pages.push(PdfPage::new(Mm(PAGE_W_MM), Mm(PAGE_H_MM), page_ops));
    }
    if pages.is_empty() {
        pages.push(PdfPage::new(Mm(PAGE_W_MM), Mm(PAGE_H_MM), vec![]));
    }

    Ok(pdf.with_pages(pages).save(&PdfSaveOptions::default(), &mut warn))
}

// ── Proper line builder ──────────────────────────────────────────────────────

fn rebuild_lines(mla: &MlaDocument, sorted_wc: &[crate::model::WorksCitedEntry]) -> Vec<DocLine> {
    use crate::model::typographical_clean;

    let mut lines: Vec<DocLine> = Vec::new();

    // Header
    for s in [
        &mla.header.student_name,
        &mla.header.instructor_name,
        &mla.header.course,
        &mla.header.date,
    ] {
        lines.push(DocLine::normal(typographical_clean(s)));
    }

    // Title (centered)
    lines.push(DocLine::centered(typographical_clean(&mla.title)));

    // Body
    for block in &mla.blocks {
        match block {
            MlaBlock::Paragraph { text, .. } => {
                let t = text.trim();
                if t.is_empty() { continue; }
                let cleaned = typographical_clean(t);
                let wrapped = word_wrap(&cleaned, text_w_mm() - INDENT_MM);
                for (i, w) in wrapped.into_iter().enumerate() {
                    if i == 0 {
                        lines.push(DocLine::paragraph(w));
                    } else {
                        lines.push(DocLine::normal(w));
                    }
                }
            }
            MlaBlock::BlockQuote { text, citation, .. } => {
                let t = text.trim();
                if t.is_empty() { continue; }
                let full = if citation.trim().is_empty() {
                    t.to_string()
                } else {
                    format!("{} {}", t, citation.trim())
                };
                let cleaned = typographical_clean(&full);
                for w in word_wrap(&cleaned, text_w_mm() - BLOCKQUOTE_INDENT_MM) {
                    lines.push(DocLine::blockquote(w));
                }
            }
            MlaBlock::SectionHeading { level, text, .. } => {
                let t = text.trim();
                if t.is_empty() { continue; }
                let cleaned = typographical_clean(t);
                match level {
                    1 => lines.push(DocLine::bold(cleaned)),
                    _ => lines.push(DocLine::centered(cleaned)),
                }
            }
        }
    }

    // Works Cited page
    lines.push(DocLine::page_break());
    let wc_title = if sorted_wc.len() == 1 { "Work Cited" } else { "Works Cited" };
    lines.push(DocLine::centered(wc_title));
    if sorted_wc.is_empty() {
        lines.push(DocLine::normal("No entries yet."));
    } else {
        for entry in sorted_wc {
            let plain = strip_markdown(&entry.format_markdown());
            let cleaned = typographical_clean(&plain);
            let wrapped = word_wrap(&cleaned, text_w_mm());
            for (i, w) in wrapped.into_iter().enumerate() {
                if i == 0 {
                    lines.push(DocLine::normal(w));
                } else {
                    lines.push(DocLine::blockquote(w)); // hanging indent
                }
            }
        }
    }

    lines
}

// ── Running head ─────────────────────────────────────────────────────────────

fn add_running_head(ops: &mut Vec<Op>, font: &FontId, last_name: &str, page_num: u32) {
    let text = format!("{} {}", last_name, page_num);
    // Right-align: approximate x so right edge of text = right margin
    let approx_w_mm = text.len() as f32 * char_w_mm();
    let x_mm = (PAGE_W_MM - MARGIN_MM - approx_w_mm).max(MARGIN_MM);
    ops.push(Op::StartTextSection);
    ops.push(Op::SetTextCursor { pos: Point::new(Mm(x_mm), Mm(running_head_y_mm())) });
    ops.push(Op::SetFont { font: PdfFontHandle::External(font.clone()), size: Pt(FONT_PT) });
    ops.push(Op::SetLineHeight { lh: Pt(FONT_PT) });
    ops.push(Op::ShowText { items: vec![PdfTextItem::Text(text)] });
    ops.push(Op::EndTextSection);
}

// ── Footnotes footer (at bottom of body page, above bottom margin) ───────────

fn add_footnotes_footer(ops: &mut Vec<Op>, font: &FontId, notes: &[crate::model::ExplanatoryNote]) {
    use crate::model::typographical_clean;

    if notes.is_empty() {
        return;
    }

    let foot_line_h_mm = 4.2;
    let total_foot_h = notes.len() as f32 * foot_line_h_mm + 5.0;
    let divider_y = MARGIN_MM + total_foot_h;

    // Standard MLA 1.5-inch rule divider (38.1 mm) rendered with standard underscore characters
    ops.push(Op::StartTextSection);
    ops.push(Op::SetTextCursor { pos: Point::new(Mm(MARGIN_MM), Mm(divider_y)) });
    ops.push(Op::SetFont { font: PdfFontHandle::External(font.clone()), size: Pt(10.0) });
    ops.push(Op::ShowText { items: vec![PdfTextItem::Text("______________________".to_string())] });
    ops.push(Op::EndTextSection);

    let mut cur_y = divider_y - 4.0;
    for note in notes {
        let text = format!("{}. {}", note.index, typographical_clean(&note.text));
        ops.push(Op::StartTextSection);
        ops.push(Op::SetTextCursor { pos: Point::new(Mm(MARGIN_MM + INDENT_MM), Mm(cur_y)) });
        ops.push(Op::SetFont { font: PdfFontHandle::External(font.clone()), size: Pt(10.0) });
        ops.push(Op::SetLineHeight { lh: Pt(12.0) });
        ops.push(Op::ShowText { items: vec![PdfTextItem::Text(text)] });
        ops.push(Op::EndTextSection);
        cur_y -= foot_line_h_mm;
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Word-wrap text to fit within `max_w_mm` using the approximate char width
fn word_wrap(text: &str, max_w_mm: f32) -> Vec<String> {
    let max_chars = ((max_w_mm / char_w_mm()) as usize).max(20);
    let mut lines = Vec::new();
    for para in text.split('\n') {
        let words: Vec<&str> = para.split_whitespace().collect();
        if words.is_empty() {
            lines.push(String::new());
            continue;
        }
        let mut cur = String::new();
        for word in words {
            if cur.is_empty() {
                cur.push_str(word);
            } else if cur.len() + 1 + word.len() <= max_chars {
                cur.push(' ');
                cur.push_str(word);
            } else {
                lines.push(cur);
                cur = word.to_string();
            }
        }
        if !cur.is_empty() {
            lines.push(cur);
        }
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

/// Strip basic markdown (*italic*, **bold**) to plain text for PDF rendering
fn strip_markdown(s: &str) -> String {
    s.replace("**", "").replace('*', "")
}
