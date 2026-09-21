use super::works_cited::WorksCitedEntry;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MlaBlock {
    Paragraph {
        id: String,
        text: String,
    },
    BlockQuote {
        id: String,
        text: String,
        citation: String,
    },
    SectionHeading {
        id: String,
        level: u8, // 1 = Bold flush left, 2 = Italics flush left, 3 = Bold centered
        text: String,
    },
}

impl MlaBlock {
    pub fn id(&self) -> &str {
        match self {
            MlaBlock::Paragraph { id, .. } => id,
            MlaBlock::BlockQuote { id, .. } => id,
            MlaBlock::SectionHeading { id, .. } => id,
        }
    }

    pub fn text(&self) -> &str {
        match self {
            MlaBlock::Paragraph { text, .. } => text,
            MlaBlock::BlockQuote { text, .. } => text,
            MlaBlock::SectionHeading { text, .. } => text,
        }
    }

    pub fn text_mut(&mut self) -> &mut String {
        match self {
            MlaBlock::Paragraph { text, .. } => text,
            MlaBlock::BlockQuote { text, .. } => text,
            MlaBlock::SectionHeading { text, .. } => text,
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            MlaBlock::Paragraph { .. } => "Paragraph",
            MlaBlock::BlockQuote { .. } => "Block Quote",
            MlaBlock::SectionHeading { level, .. } => match level {
                1 => "Heading 1 (Bold)",
                2 => "Heading 2 (Italic)",
                _ => "Heading 3 (Centered)",
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MlaHeader {
    pub student_name: String,
    pub instructor_name: String,
    pub course: String,
    pub date: String,
    pub running_header_last_name: String,
}

impl Default for MlaHeader {
    fn default() -> Self {
        Self {
            student_name: String::new(),
            instructor_name: String::new(),
            course: String::new(),
            date: format_current_mla_date(),
            running_header_last_name: String::new(),
        }
    }
}

impl MlaHeader {
    pub fn derived_last_name(&self) -> String {
        if !self.running_header_last_name.trim().is_empty() {
            return self.running_header_last_name.trim().to_string();
        }
        let parts: Vec<&str> = self.student_name.split_whitespace().collect();
        parts.last().copied().unwrap_or("").to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MlaDocument {
    pub header: MlaHeader,
    pub title: String,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub blocks: Vec<MlaBlock>,
    pub works_cited: Vec<WorksCitedEntry>,
    #[serde(default)]
    pub file_path: Option<String>,
    #[serde(default)]
    pub is_dirty: bool,
}

impl Default for MlaDocument {
    fn default() -> Self {
        Self::new_blank()
    }
}

impl MlaDocument {
    pub fn new_blank() -> Self {
        Self {
            header: MlaHeader::default(),
            title: String::new(),
            body: String::new(),
            blocks: Vec::new(),
            works_cited: Vec::new(),
            file_path: None,
            is_dirty: false,
        }
    }

    pub fn sample_template() -> Self {
        let starter_text = "Writing an academic paper in accordance with Modern Language Association (MLA) 9th edition standards requires meticulous adherence to structural formatting. Every element of the document—from the one-inch margins to the standardized double spacing—serves to establish academic rigor and uniformity across scholarly discourse. This text editor is engineered specifically to eliminate formatting errors at the source, constraining input so that margin variations, irregular paragraph spacing, and inconsistent typefaces are impossible to introduce.\n\nIn MLA format, all body paragraphs must be indented exactly one-half inch (0.5 in.) from the left margin, without any extra vertical blank space between paragraphs. The running head in the upper right-hand corner displays the author's last name followed by a single space and the current page number, positioned one-half inch from the top edge and flush with the right margin (Smith 42).\n\nWhen quoting prose that extends beyond four lines, or verse that extends beyond three lines, the quotation must be set off as a block quotation. It begins on a new line, is indented an additional half-inch from the left margin, maintains strict double line spacing, omits quotation marks, and places the parenthetical citation outside the concluding punctuation mark (MLA Handbook 120).\n\nThe Works Cited list begins on a separate page at the conclusion of the manuscript. Entries are formatted with a hanging indent of one-half inch, alphabetized by author or primary title, and constructed using the nine core container elements prescribed in the MLA ninth edition.";

        let mut doc = Self {
            header: MlaHeader {
                student_name: "Jane Doe".to_string(),
                instructor_name: "Professor Smith".to_string(),
                course: "ENG 101: Literature & Composition".to_string(),
                date: format_current_mla_date(),
                running_header_last_name: "Doe".to_string(),
            },
            title: "The Rhetorical Construction of Identity in Modern Literature".to_string(),
            body: starter_text.to_string(),
            blocks: Vec::new(),
            works_cited: Vec::new(),
            file_path: None,
            is_dirty: false,
        };

        doc.sync_blocks_from_body();

        // Sample works cited entry
        let mut entry = WorksCitedEntry::new_empty();
        entry.author = "Modern Language Association of America".to_string();
        entry.title_of_source = "MLA Handbook".to_string();
        entry.source_type = super::works_cited::SourceType::BookOrStandalone;
        entry.version = "9th ed.".to_string();
        entry.publisher = "Modern Language Association of America".to_string();
        entry.pub_date = "2021".to_string();
        doc.works_cited.push(entry);

        doc
    }

    pub fn ensure_body_synced(&mut self) {
        if self.body.trim().is_empty() && !self.blocks.is_empty() {
            let parts: Vec<String> = self
                .blocks
                .iter()
                .map(|b| match b {
                    MlaBlock::Paragraph { text, .. } => text.clone(),
                    MlaBlock::BlockQuote { text, citation, .. } => {
                        if citation.trim().is_empty() {
                            format!("> {}", text)
                        } else {
                            format!("> {} {}", text, citation.trim())
                        }
                    }
                    MlaBlock::SectionHeading { level, text, .. } => {
                        format!("{} {}", "#".repeat(*level as usize), text)
                    }
                })
                .collect();
            self.body = parts.join("\n\n");
        }
    }

    pub fn sync_blocks_from_body(&mut self) {
        let mut new_blocks = Vec::new();
        let paragraphs = self.body.split("\n\n");
        for (i, p) in paragraphs.enumerate() {
            let trimmed = p.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Some(stripped) = trimmed.strip_prefix("> ") {
                new_blocks.push(MlaBlock::BlockQuote {
                    id: generate_block_id(i + 1),
                    text: stripped.to_string(),
                    citation: String::new(),
                });
            } else if let Some(stripped) = trimmed.strip_prefix("## ") {
                new_blocks.push(MlaBlock::SectionHeading {
                    id: generate_block_id(i + 1),
                    level: 2,
                    text: stripped.to_string(),
                });
            } else if let Some(stripped) = trimmed.strip_prefix("# ") {
                new_blocks.push(MlaBlock::SectionHeading {
                    id: generate_block_id(i + 1),
                    level: 1,
                    text: stripped.to_string(),
                });
            } else {
                new_blocks.push(MlaBlock::Paragraph {
                    id: generate_block_id(i + 1),
                    text: trimmed.to_string(),
                });
            }
        }
        if new_blocks.is_empty() {
            new_blocks.push(MlaBlock::Paragraph {
                id: generate_block_id(1),
                text: String::new(),
            });
        }
        self.blocks = new_blocks;
    }

    pub fn total_word_count(&self) -> usize {
        let mut count = 0;
        count += count_words(&self.title);
        for block in &self.blocks {
            count += count_words(block.text());
            if let MlaBlock::BlockQuote { citation, .. } = block {
                count += count_words(citation);
            }
        }
        for wc in &self.works_cited {
            count += count_words(&wc.format_markdown());
        }
        count
    }

    pub fn total_char_count(&self) -> usize {
        let mut count = 0;
        count += self.title.chars().count();
        for block in &self.blocks {
            count += block.text().chars().count();
        }
        count
    }

    /// Estimated page count based on ~250 words per standard double-spaced MLA page,
    /// plus Works Cited on its own page if non-empty.
    pub fn estimated_page_count(&self) -> usize {
        let body_words: usize = self.blocks.iter().map(|b| count_words(b.text())).sum();
        let body_pages = ((body_words as f32) / 250.0).ceil() as usize;
        let body_pages = if body_pages == 0 { 1 } else { body_pages };
        if !self.works_cited.is_empty() {
            body_pages + 1
        } else {
            body_pages
        }
    }

    pub fn add_paragraph(&mut self, after_index: Option<usize>) -> usize {
        let new_block = MlaBlock::Paragraph {
            id: generate_block_id(self.blocks.len() + 1),
            text: String::new(),
        };
        self.is_dirty = true;
        match after_index {
            Some(idx) if idx < self.blocks.len() => {
                self.blocks.insert(idx + 1, new_block);
                idx + 1
            }
            _ => {
                self.blocks.push(new_block);
                self.blocks.len() - 1
            }
        }
    }

    pub fn add_blockquote(&mut self, after_index: Option<usize>) -> usize {
        let new_block = MlaBlock::BlockQuote {
            id: generate_block_id(self.blocks.len() + 1),
            text: String::new(),
            citation: String::new(),
        };
        self.is_dirty = true;
        match after_index {
            Some(idx) if idx < self.blocks.len() => {
                self.blocks.insert(idx + 1, new_block);
                idx + 1
            }
            _ => {
                self.blocks.push(new_block);
                self.blocks.len() - 1
            }
        }
    }

    pub fn add_heading(&mut self, level: u8, after_index: Option<usize>) -> usize {
        let new_block = MlaBlock::SectionHeading {
            id: generate_block_id(self.blocks.len() + 1),
            level: level.clamp(1, 3),
            text: String::new(),
        };
        self.is_dirty = true;
        match after_index {
            Some(idx) if idx < self.blocks.len() => {
                self.blocks.insert(idx + 1, new_block);
                idx + 1
            }
            _ => {
                self.blocks.push(new_block);
                self.blocks.len() - 1
            }
        }
    }

    pub fn remove_block(&mut self, index: usize) {
        if self.blocks.len() > 1 && index < self.blocks.len() {
            self.blocks.remove(index);
            self.is_dirty = true;
        }
    }

    pub fn move_block_up(&mut self, index: usize) {
        if index > 0 && index < self.blocks.len() {
            self.blocks.swap(index, index - 1);
            self.is_dirty = true;
        }
    }

    pub fn move_block_down(&mut self, index: usize) {
        if index + 1 < self.blocks.len() {
            self.blocks.swap(index, index + 1);
            self.is_dirty = true;
        }
    }

    pub fn sort_works_cited(&mut self) {
        self.works_cited.sort_by_key(|a| a.sort_key());
        self.is_dirty = true;
    }
}

pub fn count_words(s: &str) -> usize {
    s.split_whitespace().count()
}

pub fn format_current_mla_date() -> String {
    let now = chrono::Local::now();
    // MLA date format: "Day Month Year" e.g. "20 September 2026"
    now.format("%d %B %Y").to_string()
}

/// Converts a string to MLA Title Capitalization rules:
/// - Capitalize first and last words
/// - Capitalize all principal words (nouns, verbs, adjectives, adverbs, pronouns)
/// - Do NOT capitalize articles (a, an, the), prepositions (in, on, of, for, with, at, by, to, from, under),
///   or coordinating conjunctions (and, but, or, nor, for, so, yet) unless they are the first or last word.
pub fn to_mla_title_case(input: &str) -> String {
    let words: Vec<&str> = input.split_whitespace().collect();
    if words.is_empty() {
        return String::new();
    }

    let lowercase_words = [
        "a", "an", "the", "and", "but", "or", "nor", "for", "so", "yet", "as", "at", "by", "for",
        "from", "in", "into", "of", "off", "on", "onto", "out", "over", "to", "up", "with", "vs",
        "via", "than",
    ];

    let total = words.len();
    let result: Vec<String> = words
        .iter()
        .enumerate()
        .map(|(i, &word)| {
            let lower = word.to_lowercase();
            let is_after_colon = if i > 0 {
                let prev = words[i - 1];
                prev.ends_with(':') || prev.ends_with(';')
            } else {
                false
            };

            // Check if hyphenated
            if lower.contains('-') {
                let parts: Vec<String> = lower.split('-').map(capitalize_first).collect();
                parts.join("-")
            } else if (i == 0 || i == total - 1 || is_after_colon)
                || !lowercase_words.contains(&lower.as_str())
            {
                capitalize_first(&lower)
            } else {
                lower
            }
        })
        .collect();

    result.join(" ")
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

fn generate_block_id(seed: usize) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("blk_{}_{:x}", seed, nanos)
}
