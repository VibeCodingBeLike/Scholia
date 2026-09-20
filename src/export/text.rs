use crate::model::{MlaBlock, MlaDocument};
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub fn export_to_text(doc: &MlaDocument, path: &Path) -> Result<(), String> {
    let text_content = generate_mla_text(doc);
    let mut file = File::create(path).map_err(|e| format!("Failed to create text file: {}", e))?;
    file.write_all(text_content.as_bytes())
        .map_err(|e| format!("Failed to write text: {}", e))?;
    Ok(())
}

pub fn generate_mla_text(doc: &MlaDocument) -> String {
    let mut out = String::new();
    let last_name = doc.header.derived_last_name();

    // Running head for page 1
    out.push_str(&format!("{:>70}\n\n", format!("{} 1", last_name)));

    // Header lines
    out.push_str(&format!("{}\n", doc.header.student_name));
    out.push_str(&format!("{}\n", doc.header.instructor_name));
    out.push_str(&format!("{}\n", doc.header.course));
    out.push_str(&format!("{}\n\n", doc.header.date));

    // Centered Title (approx 70 columns)
    let title = &doc.title;
    let padding = if title.len() < 70 {
        (70 - title.len()) / 2
    } else {
        0
    };
    out.push_str(&format!("{}{}\n\n", " ".repeat(padding), title));

    // Blocks
    for block in &doc.blocks {
        match block {
            MlaBlock::Paragraph { text, .. } => {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    // 0.5 in indent is typically 5 spaces in plain text
                    out.push_str(&format!("     {}\n\n", trimmed));
                }
            }
            MlaBlock::BlockQuote { text, citation, .. } => {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    let cite = if citation.trim().is_empty() {
                        String::new()
                    } else {
                        format!(" {}", citation.trim())
                    };
                    out.push_str(&format!("          {}{}\n\n", trimmed, cite));
                }
            }
            MlaBlock::SectionHeading { level, text, .. } => {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    match level {
                        1 => out.push_str(&format!("{}\n\n", trimmed.to_uppercase())),
                        2 => out.push_str(&format!("*{}*\n\n", trimmed)),
                        _ => {
                            let pad = if trimmed.len() < 70 {
                                (70 - trimmed.len()) / 2
                            } else {
                                0
                            };
                            out.push_str(&format!("{}{}\n\n", " ".repeat(pad), trimmed));
                        }
                    }
                }
            }
        }
    }

    // Works Cited
    if !doc.works_cited.is_empty() {
        out.push_str(
            "\n\n----------------------------------------------------------------------\n\n",
        );
        let wc_title = if doc.works_cited.len() == 1 {
            "Work Cited"
        } else {
            "Works Cited"
        };
        let pad = (70 - wc_title.len()) / 2;
        out.push_str(&format!("{}{}\n\n", " ".repeat(pad), wc_title));

        let mut sorted = doc.works_cited.clone();
        sorted.sort_by_key(|a| a.sort_key());

        for entry in &sorted {
            out.push_str(&format!("{}\n\n", entry.format_markdown()));
        }
    }

    out
}
