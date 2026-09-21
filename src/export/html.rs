use crate::model::{MlaBlock, MlaDocument};
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub fn export_to_html(doc: &MlaDocument, path: &Path) -> Result<(), String> {
    let html_content = generate_mla_html(doc);
    let mut file = File::create(path).map_err(|e| format!("Failed to create HTML file: {}", e))?;
    file.write_all(html_content.as_bytes())
        .map_err(|e| format!("Failed to write HTML: {}", e))?;
    Ok(())
}

pub fn generate_mla_html(doc: &MlaDocument) -> String {
    let last_name = doc.header.derived_last_name();
    let mut sorted_wc = doc.works_cited.clone();
    sorted_wc.sort_by_key(|a| a.sort_key());

    let mut body_html = String::new();

    // First page heading
    body_html.push_str(&format!(
        r#"<div class="mla-header">
    <p class="header-line">{}</p>
    <p class="header-line">{}</p>
    <p class="header-line">{}</p>
    <p class="header-line">{}</p>
</div>"#,
        escape_html(&doc.header.student_name),
        escape_html(&doc.header.instructor_name),
        escape_html(&doc.header.course),
        escape_html(&doc.header.date),
    ));

    // Title
    body_html.push_str(&format!(
        r#"<h1 class="mla-title">{}</h1>"#,
        escape_html(&doc.title)
    ));

    // Blocks
    for block in &doc.blocks {
        let block_notes = doc.notes_for_block(block.id());

        match block {
            MlaBlock::Paragraph { text, .. } => {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    let text_with_notes = crate::model::render_text_with_note_tags(
                        trimmed,
                        block.note_tags(),
                        &block_notes,
                        |idx| format!("<sup>{}</sup>", idx),
                    );
                    body_html.push_str(&format!(
                        r#"<p class="mla-paragraph">{}</p>"#,
                        markdown_to_html(&text_with_notes)
                    ));
                }
            }
            MlaBlock::BlockQuote { text, citation, .. } => {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    let text_with_notes = crate::model::render_text_with_note_tags(
                        trimmed,
                        block.note_tags(),
                        &block_notes,
                        |idx| format!("<sup>{}</sup>", idx),
                    );
                    let cite_part = if citation.trim().is_empty() {
                        String::new()
                    } else {
                        format!(" {}", escape_html(citation.trim()))
                    };
                    body_html.push_str(&format!(
                        r#"<blockquote class="mla-blockquote">{}<span class="citation">{}</span></blockquote>"#,
                        markdown_to_html(&text_with_notes),
                        cite_part
                    ));
                }
            }
            MlaBlock::SectionHeading { level, text, .. } => {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    let class_name = match level {
                        1 => "mla-h1",
                        2 => "mla-h2",
                        _ => "mla-h3",
                    };
                    body_html.push_str(&format!(
                        r#"<div class="{}">{}</div>"#,
                        class_name,
                        escape_html(trimmed)
                    ));
                }
            }
        }
    }

    // Footnotes / Notes Section (Rendered in footer at bottom of body text per MLA standards)
    if !doc.notes.is_empty() {
        body_html.push_str(
            r#"<footer class="mla-footnotes">
    <hr class="mla-footnotes-divider">"#,
        );
        for note in &doc.notes {
            let note_str = format!("{}. {}", note.index, crate::model::typographical_clean(&note.text));
            body_html.push_str(&format!(
                r#"    <p class="mla-footnote-entry">{}</p>"#,
                escape_html(&note_str)
            ));
        }
        body_html.push_str("</footer>");
    }

    // Works Cited
    let title_label = if sorted_wc.len() == 1 {
        "Work Cited"
    } else {
        "Works Cited"
    };
    body_html.push_str(&format!(
        r#"<div class="mla-works-cited-section">
    <h2 class="mla-works-cited-title">{}</h2>"#,
        title_label
    ));

    if sorted_wc.is_empty() {
        body_html.push_str(r#"    <p class="mla-works-cited-entry" style="text-align: center; text-indent: 0;">No entries yet.</p>"#);
    } else {
        for entry in &sorted_wc {
            let md = entry.format_markdown();
            let cleaned = crate::model::typographical_clean(&md);
            body_html.push_str(&format!(
                r#"    <p class="mla-works-cited-entry">{}</p>"#,
                markdown_to_html(&cleaned)
            ));
        }
    }

    body_html.push_str("</div>");

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <style>
        /* Modern MLA 9th Edition Standard Stylesheet */
        @page {{
            size: letter;
            margin: 1in;
            @top-right {{
                content: "{last_name} " counter(page);
                font-family: 'Times New Roman', Times, serif;
                font-size: 12pt;
            }}
        }}

        body {{
            font-family: 'Times New Roman', Times, serif;
            font-size: 12pt;
            line-height: 2.0; /* Strict MLA double spacing */
            color: #000;
            background-color: #fafafa;
            margin: 0;
            padding: 20px 0;
        }}

        .print-banner {{
            max-width: 8.5in;
            margin: 0 auto 20px auto;
            background: #2b303c;
            color: #fff;
            padding: 12px 24px;
            border-radius: 8px;
            display: flex;
            justify-content: space-between;
            align-items: center;
            font-family: system-ui, -apple-system, sans-serif;
            font-size: 14px;
            box-shadow: 0 4px 12px rgba(0,0,0,0.15);
        }}

        .print-btn {{
            background: #2563eb;
            color: white;
            border: none;
            padding: 8px 18px;
            font-size: 14px;
            font-weight: bold;
            border-radius: 6px;
            cursor: pointer;
            transition: background 0.2s;
        }}
        .print-btn:hover {{
            background: #1d4ed8;
        }}

        .page-sheet {{
            background: #ffffff;
            width: 8.5in;
            min-height: 11in;
            padding: 1in;
            margin: 0 auto;
            box-sizing: border-box;
            box-shadow: 0 4px 20px rgba(0, 0, 0, 0.08);
            position: relative;
        }}

        .running-head-screen {{
            text-align: right;
            margin-bottom: 0.5in;
            margin-top: -0.5in;
            font-size: 12pt;
        }}

        .mla-header {{
            margin-bottom: 0;
        }}
        .header-line {{
            margin: 0;
            padding: 0;
            text-indent: 0;
        }}

        .mla-title {{
            font-size: 12pt;
            font-weight: normal;
            text-align: center;
            margin: 0;
            padding: 0;
        }}

        .mla-paragraph {{
            text-indent: 0.5in;
            margin: 0;
            padding: 0;
            text-align: left;
        }}

        .mla-blockquote {{
            margin-left: 0.5in;
            margin-right: 0;
            margin-top: 0;
            margin-bottom: 0;
            padding: 0;
            text-indent: 0;
            border: none;
        }}

        .mla-h1 {{
            font-weight: bold;
            text-align: left;
            margin: 0;
            padding: 0;
        }}
        .mla-h2 {{
            font-style: italic;
            text-align: left;
            margin: 0;
            padding: 0;
        }}
        .mla-h3 {{
            font-weight: bold;
            text-align: center;
            margin: 0;
            padding: 0;
        }}

        .mla-footnotes {{
            margin-top: 2rem;
            margin-bottom: 1rem;
        }}

        .mla-footnotes-divider {{
            width: 1.5in;
            border: none;
            border-top: 1px solid #000;
            margin: 0 0 0.5rem 0;
            text-align: left;
        }}

        .mla-footnote-entry {{
            font-size: 10pt;
            line-height: 1.4;
            text-indent: 0.5in;
            margin: 0 0 0.25rem 0;
            padding: 0;
            text-align: left;
        }}

        .mla-works-cited-section {{
            page-break-before: always;
            margin-top: 0;
        }}

        .mla-works-cited-title {{
            font-size: 12pt;
            font-weight: normal;
            text-align: center;
            margin: 0;
            padding: 0;
        }}

        .mla-works-cited-entry {{
            margin: 0;
            padding: 0;
            padding-left: 0.5in;
            text-indent: -0.5in; /* Hanging indent */
        }}

        @media print {{
            body {{
                background: none;
                padding: 0;
            }}
            .print-banner {{
                display: none;
            }}
            .page-sheet {{
                width: 100%;
                min-height: auto;
                padding: 0;
                margin: 0;
                box-shadow: none;
            }}
            .running-head-screen {{
                display: none;
            }}
            .mla-works-cited-section {{
                page-break-before: always;
            }}
        }}
    </style>
</head>
<body>
    <div class="print-banner">
        <span><strong>Scholia</strong> &bull; Print Preview (Exact MLA 9th Edition Standard)</span>
        <button class="print-btn" onclick="window.print()">Print / Save as PDF</button>
    </div>

    <div class="page-sheet">
        <div class="running-head-screen">{last_name} 1</div>
        {body_html}
    </div>
</body>
</html>"#,
        escape_html(&doc.title),
    )
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn markdown_to_html(s: &str) -> String {
    let mut out = String::new();
    let parts: Vec<&str> = s.split('*').collect();
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        let escaped = escape_html(part);
        if i % 2 == 1 {
            out.push_str(&format!("<em>{}</em>", escaped));
        } else {
            out.push_str(&escaped);
        }
    }
    out
}
