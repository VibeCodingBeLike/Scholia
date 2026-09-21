use super::works_cited::WorksCitedEntry;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoteTag {
    pub note_index: usize,
    #[serde(default)]
    pub word: String,
    #[serde(default)]
    pub offset: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MlaBlock {
    Paragraph {
        id: String,
        text: String,
        #[serde(default)]
        note_tags: Vec<NoteTag>,
    },
    BlockQuote {
        id: String,
        text: String,
        citation: String,
        #[serde(default)]
        note_tags: Vec<NoteTag>,
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

    pub fn note_tags(&self) -> &[NoteTag] {
        match self {
            MlaBlock::Paragraph { note_tags, .. } => note_tags,
            MlaBlock::BlockQuote { note_tags, .. } => note_tags,
            MlaBlock::SectionHeading { .. } => &[],
        }
    }

    pub fn note_tags_mut(&mut self) -> Option<&mut Vec<NoteTag>> {
        match self {
            MlaBlock::Paragraph { note_tags, .. } => Some(note_tags),
            MlaBlock::BlockQuote { note_tags, .. } => Some(note_tags),
            MlaBlock::SectionHeading { .. } => None,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExplanatoryNote {
    pub id: String,
    pub index: usize,
    pub text: String,
    #[serde(default)]
    pub block_id: String,
    #[serde(default)]
    pub word: String,
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
    pub notes: Vec<ExplanatoryNote>,
    #[serde(default)]
    pub file_path: Option<String>,
    #[serde(default)]
    pub is_dirty: bool,
    #[serde(skip)]
    pub active_block_idx: usize,
    #[serde(skip)]
    pub requested_focus_block_idx: Option<usize>,
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
            blocks: vec![MlaBlock::Paragraph {
                id: generate_block_id(1),
                text: String::new(),
                note_tags: Vec::new(),
            }],
            works_cited: Vec::new(),
            notes: Vec::new(),
            file_path: None,
            is_dirty: false,
            active_block_idx: 0,
            requested_focus_block_idx: None,
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
            notes: Vec::new(),
            file_path: None,
            is_dirty: false,
            active_block_idx: 0,
            requested_focus_block_idx: None,
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

    pub fn sync_body_from_blocks(&mut self) {
        let parts: Vec<String> = self
            .blocks
            .iter()
            .map(|b| match b {
                MlaBlock::Paragraph { text, .. } => text.clone(),
                MlaBlock::BlockQuote { text, citation, .. } => {
                    if citation.trim().is_empty() {
                        format!("> {}", text.trim())
                    } else {
                        format!("> {} {}", text.trim(), citation.trim())
                    }
                }
                MlaBlock::SectionHeading { level, text, .. } => {
                    format!("{} {}", "#".repeat(*level as usize), text.trim())
                }
            })
            .collect();
        self.body = parts.join("\n\n");
    }

    pub fn ensure_blocks_initialized(&mut self) {
        if self.blocks.is_empty() {
            if !self.body.trim().is_empty() {
                self.sync_blocks_from_body();
            } else {
                self.blocks.push(MlaBlock::Paragraph {
                    id: generate_block_id(1),
                    text: String::new(),
                    note_tags: Vec::new(),
                });
            }
        }
    }

    pub fn ensure_body_synced(&mut self) {
        if self.body.trim().is_empty() && !self.blocks.is_empty() {
            self.sync_body_from_blocks();
        }
    }

    pub fn sync_blocks_from_body(&mut self) {
        let mut new_blocks = Vec::new();
        for (i, p) in self.body.lines().enumerate() {
            let trimmed = p.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Some(stripped) = trimmed.strip_prefix("> ") {
                new_blocks.push(MlaBlock::BlockQuote {
                    id: generate_block_id(i + 1),
                    text: stripped.to_string(),
                    citation: String::new(),
                    note_tags: Vec::new(),
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
                    note_tags: Vec::new(),
                });
            }
        }
        if new_blocks.is_empty() {
            new_blocks.push(MlaBlock::Paragraph {
                id: generate_block_id(1),
                text: String::new(),
                note_tags: Vec::new(),
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
    /// plus the Works Cited on its own separate page.
    pub fn estimated_page_count(&self) -> usize {
        let body_words: usize = self.blocks.iter().map(|b| count_words(b.text())).sum();
        let body_pages = ((body_words as f32) / 250.0).ceil() as usize;
        let body_pages = if body_pages == 0 { 1 } else { body_pages };
        // The Works Cited list is always formatted on its own separate page at the conclusion of the manuscript
        body_pages + 1
    }

    pub fn add_paragraph(&mut self, after_index: Option<usize>) -> usize {
        let new_block = MlaBlock::Paragraph {
            id: generate_block_id(self.blocks.len() + 1),
            text: String::new(),
            note_tags: Vec::new(),
        };
        self.is_dirty = true;
        let idx = match after_index {
            Some(idx) if idx < self.blocks.len() => {
                self.blocks.insert(idx + 1, new_block);
                idx + 1
            }
            _ => {
                self.blocks.push(new_block);
                self.blocks.len() - 1
            }
        };
        self.sync_body_from_blocks();
        idx
    }

    pub fn add_blockquote(&mut self, after_index: Option<usize>) -> usize {
        let new_block = MlaBlock::BlockQuote {
            id: generate_block_id(self.blocks.len() + 1),
            text: String::new(),
            citation: String::new(),
            note_tags: Vec::new(),
        };
        self.is_dirty = true;
        let idx = match after_index {
            Some(idx) if idx < self.blocks.len() => {
                self.blocks.insert(idx + 1, new_block);
                idx + 1
            }
            _ => {
                self.blocks.push(new_block);
                self.blocks.len() - 1
            }
        };
        self.sync_body_from_blocks();
        idx
    }

    pub fn add_heading(&mut self, level: u8, after_index: Option<usize>) -> usize {
        let new_block = MlaBlock::SectionHeading {
            id: generate_block_id(self.blocks.len() + 1),
            level: level.clamp(1, 3),
            text: String::new(),
        };
        self.is_dirty = true;
        let idx = match after_index {
            Some(idx) if idx < self.blocks.len() => {
                self.blocks.insert(idx + 1, new_block);
                idx + 1
            }
            _ => {
                self.blocks.push(new_block);
                self.blocks.len() - 1
            }
        };
        self.sync_body_from_blocks();
        idx
    }

    pub fn remove_block(&mut self, index: usize) {
        if self.blocks.len() > 1 && index < self.blocks.len() {
            let removed_id = self.blocks[index].id().to_string();
            self.blocks.remove(index);
            self.notes.retain(|n| n.block_id != removed_id);
            self.reindex_notes();
            self.is_dirty = true;
            self.sync_body_from_blocks();
        }
    }

    pub fn move_block_up(&mut self, index: usize) {
        if index > 0 && index < self.blocks.len() {
            self.blocks.swap(index, index - 1);
            self.reindex_notes();
            self.is_dirty = true;
            self.sync_body_from_blocks();
        }
    }

    pub fn move_block_down(&mut self, index: usize) {
        if index + 1 < self.blocks.len() {
            self.blocks.swap(index, index + 1);
            self.reindex_notes();
            self.is_dirty = true;
            self.sync_body_from_blocks();
        }
    }

    pub fn sort_works_cited(&mut self) {
        self.works_cited.sort_by_key(|a| a.sort_key());
        self.is_dirty = true;
    }

    pub fn add_explanatory_note(&mut self, text: String) -> usize {
        self.ensure_blocks_initialized();
        let b_idx = self.active_block_idx.min(self.blocks.len().saturating_sub(1));
        let block_id = self.blocks[b_idx].id().to_string();
        self.add_note_to_block(&block_id, text)
    }

    pub fn add_note_to_block(&mut self, block_id: &str, text: String) -> usize {
        let next_idx = self.notes.len() + 1;
        self.notes.push(ExplanatoryNote {
            id: generate_block_id(next_idx),
            index: next_idx,
            text,
            block_id: block_id.to_string(),
            word: String::new(),
        });
        self.reindex_notes();
        self.is_dirty = true;
        self.notes
            .iter()
            .rposition(|n| n.block_id == block_id)
            .map(|pos| self.notes[pos].index)
            .unwrap_or(next_idx)
    }

    pub fn delete_explanatory_note(&mut self, index: usize) {
        if let Some(pos) = self.notes.iter().position(|n| n.index == index) {
            let note_id = self.notes[pos].id.clone();
            self.delete_note_by_id(&note_id);
        }
    }

    pub fn delete_note_by_id(&mut self, id: &str) {
        if let Some(pos) = self.notes.iter().position(|n| n.id == id) {
            let deleted_idx = self.notes[pos].index;
            self.notes.remove(pos);

            for block in &mut self.blocks {
                if let Some(tags) = block.note_tags_mut() {
                    tags.retain(|t| t.note_index != deleted_idx);
                }
            }

            self.reindex_notes();
            self.is_dirty = true;
        }
    }

    pub fn link_note_from_shortcut(&mut self, block_id: &str, note_index: usize, word: &str) {
        if let Some(note) = self.notes.iter_mut().find(|n| n.index == note_index) {
            note.block_id = block_id.to_string();
            if !word.is_empty() {
                note.word = word.to_string();
            }
        } else {
            self.notes.push(ExplanatoryNote {
                id: generate_block_id(note_index),
                index: note_index,
                text: String::new(),
                block_id: block_id.to_string(),
                word: word.to_string(),
            });
            self.reindex_notes();
        }
        self.is_dirty = true;
    }

    pub fn notes_for_block(&self, block_id: &str) -> Vec<&ExplanatoryNote> {
        self.notes.iter().filter(|n| n.block_id == block_id).collect()
    }

    /// Re-indexes all notes sequentially based on document block order, keeping note_tags in lockstep.
    pub fn reindex_notes(&mut self) {
        let block_order: std::collections::HashMap<&str, usize> = self
            .blocks
            .iter()
            .enumerate()
            .map(|(i, b)| (b.id(), i))
            .collect();

        self.notes.sort_by_key(|n| {
            (
                block_order.get(n.block_id.as_str()).copied().unwrap_or(usize::MAX),
                n.index,
            )
        });

        let mut old_to_new = std::collections::HashMap::new();
        for (i, note) in self.notes.iter_mut().enumerate() {
            let new_index = i + 1;
            old_to_new.insert(note.index, new_index);
            note.index = new_index;
        }

        for block in &mut self.blocks {
            if let Some(tags) = block.note_tags_mut() {
                for tag in tags.iter_mut() {
                    if let Some(&new_idx) = old_to_new.get(&tag.note_index) {
                        tag.note_index = new_idx;
                    }
                }
            }
        }
    }

    pub fn toggle_block_type(&mut self, idx: usize) {
        if idx < self.blocks.len() {
            match &self.blocks[idx] {
                MlaBlock::Paragraph { id, text, note_tags } => {
                    self.blocks[idx] = MlaBlock::BlockQuote {
                        id: id.clone(),
                        text: text.clone(),
                        citation: String::new(),
                        note_tags: note_tags.clone(),
                    };
                }
                MlaBlock::BlockQuote { id, text, citation, note_tags } => {
                    let mut combined = text.clone();
                    if !citation.trim().is_empty() {
                        combined.push(' ');
                        combined.push_str(citation.trim());
                    }
                    self.blocks[idx] = MlaBlock::Paragraph {
                        id: id.clone(),
                        text: combined,
                        note_tags: note_tags.clone(),
                    };
                }
                MlaBlock::SectionHeading { id, text, .. } => {
                    self.blocks[idx] = MlaBlock::Paragraph {
                        id: id.clone(),
                        text: text.clone(),
                        note_tags: Vec::new(),
                    };
                }
            }
            self.sync_body_from_blocks();
            self.is_dirty = true;
        }
    }

    pub fn sync_notes_with_body(&mut self) {
        self.ensure_blocks_initialized();
        let valid_block_ids: std::collections::HashSet<&str> =
            self.blocks.iter().map(|b| b.id()).collect();

        for (i, note) in self.notes.iter_mut().enumerate() {
            if note.block_id.is_empty() || !valid_block_ids.contains(note.block_id.as_str()) {
                let target_idx = i.min(self.blocks.len().saturating_sub(1));
                note.block_id = self.blocks[target_idx].id().to_string();
            }
        }

        // Clean out any raw unicode superscripts from block text
        for block in &mut self.blocks {
            let cleaned = clean_superscripts(block.text());
            if cleaned != block.text() {
                *block.text_mut() = cleaned;
            }
        }

        self.reindex_notes();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteShortcutResult {
    pub note_index: usize,
    pub word: String,
}

/// Parses shorthand note creation e.g. `word^(1)` followed by a space.
/// When user hits spacebar to confirm the note, `^(1)` is removed without deleting the associated word,
/// and a NoteTag is added to the block's `note_tags`.
pub fn try_parse_and_apply_note_shortcut(
    text: &mut String,
    note_tags: &mut Vec<NoteTag>,
) -> Option<NoteShortcutResult> {
    let open_idx = text.find("^(")?;
    let after_open = &text[open_idx + 2..];
    let close_rel = after_open.find(')')?;
    let close_idx = open_idx + 2 + close_rel;

    let digits_str = &text[open_idx + 2..close_idx];
    let note_num: usize = digits_str.parse().ok()?;
    if note_num == 0 {
        return None;
    }

    let after_close = &text[close_idx + 1..];
    if !after_close.starts_with(' ') && !after_close.starts_with('\t') && !after_close.is_empty() {
        return None;
    }

    let preceding_text = &text[..open_idx];
    let word = preceding_text
        .split_whitespace()
        .last()
        .unwrap_or("")
        .trim_matches(|c: char| {
            c == '"' || c == '\'' || c == '(' || c == '[' || c == '“' || c == '”' || c == ',' || c == '.' || c == ';' || c == ':'
        })
        .to_string();

    let replace_end = if after_close.starts_with(' ') {
        close_idx + 2
    } else {
        close_idx + 1
    };

    text.replace_range(open_idx..replace_end, " ");
    let offset = open_idx;

    if let Some(existing) = note_tags.iter_mut().find(|t| t.note_index == note_num) {
        existing.word = word.clone();
        existing.offset = offset;
    } else {
        note_tags.push(NoteTag {
            note_index: note_num,
            word: word.clone(),
            offset,
        });
        note_tags.sort_by_key(|t| t.offset);
    }

    Some(NoteShortcutResult {
        note_index: note_num,
        word,
    })
}

/// Renders block text by embedding superscript representations right after the tagged words,
/// falling back to appending block notes to the end if no inline tags exist.
pub fn render_text_with_note_tags(
    text: &str,
    tags: &[NoteTag],
    fallback_notes: &[&ExplanatoryNote],
    format_sup: impl Fn(usize) -> String,
) -> String {
    if tags.is_empty() {
        if fallback_notes.is_empty() {
            return text.to_string();
        } else {
            let sups: String = fallback_notes.iter().map(|n| format_sup(n.index)).collect();
            return format!("{}{}", text, sups);
        }
    }

    let mut sorted_tags = tags.to_vec();
    // Sort descending by offset so that insertions do not shift earlier character offsets
    sorted_tags.sort_by(|a, b| b.offset.cmp(&a.offset));

    let mut result = text.to_string();
    for tag in sorted_tags {
        let sup = format_sup(tag.note_index);
        let mut inserted = false;

        if tag.offset <= result.len() && result.is_char_boundary(tag.offset) {
            let prefix = &result[..tag.offset];
            if prefix.ends_with(&tag.word) || tag.word.is_empty() {
                result.insert_str(tag.offset, &sup);
                inserted = true;
            }
        }

        if !inserted && !tag.word.is_empty() {
            if let Some(pos) = result.rfind(&tag.word) {
                let insert_pos = pos + tag.word.len();
                if result.is_char_boundary(insert_pos) {
                    result.insert_str(insert_pos, &sup);
                    inserted = true;
                }
            }
        }

        if !inserted {
            result.push_str(&sup);
        }
    }

    result
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
        "a", "an", "the", "and", "but", "or", "nor", "for", "so", "yet", "as", "at", "by",
        "from", "in", "into", "of", "off", "on", "onto", "out", "over", "to", "up", "with", "vs",
        "via", "than", "is", "if", "it", "its", "are", "be", "am", "was", "were", "per", "en",
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

            let clean = lower.trim_matches(|c: char| !c.is_alphanumeric());

            // Check if hyphenated
            if lower.contains('-') {
                let parts: Vec<String> = lower
                    .split('-')
                    .enumerate()
                    .map(|(pi, p)| {
                        let p_clean = p.trim_matches(|c: char| !c.is_alphanumeric());
                        if pi > 0 && lowercase_words.contains(&p_clean) {
                            p.to_string()
                        } else {
                            capitalize_first(p)
                        }
                    })
                    .collect();
                parts.join("-")
            } else if (i == 0 || i == total - 1 || is_after_colon)
                || !lowercase_words.contains(&clean)
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
    let mut leading = String::new();
    for c in chars.by_ref() {
        if c.is_alphanumeric() {
            let mut res = leading;
            res.extend(c.to_uppercase());
            res.push_str(chars.as_str());
            return res;
        } else {
            leading.push(c);
        }
    }
    leading
}

pub fn generate_block_id(seed: usize) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("blk_{}_{:x}", seed, nanos)
}

pub fn num_to_superscript(n: usize) -> String {
    n.to_string()
        .chars()
        .map(|c| match c {
            '0' => '⁰',
            '1' => '¹',
            '2' => '²',
            '3' => '³',
            '4' => '⁴',
            '5' => '⁵',
            '6' => '⁶',
            '7' => '⁷',
            '8' => '⁸',
            '9' => '⁹',
            other => other,
        })
        .collect()
}

pub fn is_superscript_digit(c: char) -> bool {
    matches!(c, '⁰' | '¹' | '²' | '³' | '⁴' | '⁵' | '⁶' | '⁷' | '⁸' | '⁹')
}

pub fn superscript_to_num(s: &str) -> Option<usize> {
    let mut num_str = String::new();
    for c in s.chars() {
        match c {
            '⁰' => num_str.push('0'),
            '¹' => num_str.push('1'),
            '²' => num_str.push('2'),
            '³' => num_str.push('3'),
            '⁴' => num_str.push('4'),
            '⁵' => num_str.push('5'),
            '⁶' => num_str.push('6'),
            '⁷' => num_str.push('7'),
            '⁸' => num_str.push('8'),
            '⁹' => num_str.push('9'),
            _ => return None,
        }
    }
    num_str.parse().ok()
}

pub fn clean_superscripts(s: &str) -> String {
    s.chars().filter(|c| !is_superscript_digit(*c)).collect()
}

/// Smart Typographical Cleaning on Export:
/// - Straight quotes (", ') converted to curly smart quotes (“ ”, ‘ ’)
/// - Double hyphens (--) converted to em dashes (—) without surrounding spaces
pub fn typographical_clean(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        // Double hyphens: "--" or " -- " -> "—"
        if chars[i] == '-' && i + 1 < len && chars[i + 1] == '-' {
            if result.ends_with(' ') {
                result.pop();
            }
            result.push('—');
            i += 2;
            if i < len && chars[i] == ' ' {
                i += 1;
            }
            continue;
        }

        // Double quotes: " -> “ or ”
        if chars[i] == '"' {
            let prev = if i > 0 { Some(chars[i - 1]) } else { None };
            let is_open = match prev {
                None => true,
                Some(c) if c.is_whitespace() || c == '(' || c == '[' || c == '{' || c == '—' => true,
                _ => false,
            };
            if is_open {
                result.push('“');
            } else {
                result.push('”');
            }
            i += 1;
            continue;
        }

        // Single quotes: ' -> ‘ or ’
        if chars[i] == '\'' {
            let prev = if i > 0 { Some(chars[i - 1]) } else { None };
            let next = if i + 1 < len { Some(chars[i + 1]) } else { None };

            // Apostrophe inside word: don't, Smith's
            if let (Some(p), Some(n)) = (prev, next) {
                if p.is_alphabetic() && n.is_alphabetic() {
                    result.push('’');
                    i += 1;
                    continue;
                }
            }

            let is_open = match prev {
                None => true,
                Some(c) if c.is_whitespace() || c == '(' || c == '[' || c == '{' || c == '—' => true,
                _ => false,
            };
            if is_open {
                result.push('‘');
            } else {
                result.push('’');
            }
            i += 1;
            continue;
        }

        result.push(chars[i]);
        i += 1;
    }

    result
}
