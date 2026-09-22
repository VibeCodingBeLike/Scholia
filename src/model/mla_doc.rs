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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

    pub fn content_equals(&self, other: &Self) -> bool {
        self.title == other.title
            && self.header == other.header
            && self.blocks == other.blocks
            && self.works_cited == other.works_cited
            && self.notes == other.notes
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

    pub fn move_sentence_in_active_block(
        &mut self,
        cursor_char_idx: usize,
        direction_left: bool,
    ) -> Result<usize, &'static str> {
        let block = self
            .blocks
            .get_mut(self.active_block_idx)
            .ok_or("No active block selected.")?;
        let text = block.text_mut();
        if let Some((new_text, new_cursor)) =
            reorder_sentences(text, cursor_char_idx, direction_left)
        {
            *text = new_text;
            self.is_dirty = true;
            self.sync_body_from_blocks();
            Ok(new_cursor)
        } else {
            let (_leading, spans) = split_sentences(text);
            if spans.len() <= 1 {
                Err("Only one sentence in active paragraph.")
            } else if direction_left {
                Err("Already at the first sentence.")
            } else {
                Err("Already at the last sentence.")
            }
        }
    }

    pub fn sort_works_cited(&mut self) {
        self.works_cited.sort_by_key(|a| a.sort_key());
        self.is_dirty = true;
    }

    pub fn add_explanatory_note(&mut self, text: String) -> usize {
        self.ensure_blocks_initialized();
        let b_idx = self
            .active_block_idx
            .min(self.blocks.len().saturating_sub(1));
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
            let del_sup = num_to_superscript(deleted_idx);
            self.notes.remove(pos);

            for block in &mut self.blocks {
                if block.text().contains(&del_sup) {
                    *block.text_mut() = block.text().replace(&del_sup, "");
                }
                if let Some(tags) = block.note_tags_mut() {
                    tags.retain(|t| t.note_index != deleted_idx);
                }
            }

            self.reindex_notes();
            self.is_dirty = true;
        }
    }

    pub fn existing_note_indices(&self) -> Vec<usize> {
        let mut indices: Vec<usize> = self.notes.iter().map(|n| n.index).collect();
        for block in &self.blocks {
            for tag in block.note_tags() {
                if !indices.contains(&tag.note_index) {
                    indices.push(tag.note_index);
                }
            }
        }
        indices
    }

    pub fn link_note_from_shortcut(
        &mut self,
        block_id: &str,
        note_index: usize,
        word: &str,
    ) -> bool {
        if let Some(note) = self.notes.iter_mut().find(|n| n.index == note_index) {
            if !note.word.is_empty() && note.word != word {
                // In MLA 9, reusing a note for multiple words is prohibited
                return false;
            }
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
        true
    }

    pub fn notes_for_block(&self, block_id: &str) -> Vec<&ExplanatoryNote> {
        self.notes
            .iter()
            .filter(|n| n.block_id == block_id)
            .collect()
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
                block_order
                    .get(n.block_id.as_str())
                    .copied()
                    .unwrap_or(usize::MAX),
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
            let mut replacements = Vec::new();
            if let Some(tags) = block.note_tags_mut() {
                for tag in tags.iter_mut() {
                    if let Some(&new_idx) = old_to_new.get(&tag.note_index) {
                        if tag.note_index != new_idx {
                            let old_sup = num_to_superscript(tag.note_index);
                            let new_sup = num_to_superscript(new_idx);
                            replacements.push((old_sup, new_sup));
                            tag.note_index = new_idx;
                        }
                    }
                }
            }
            for (old_sup, new_sup) in replacements {
                let updated = block.text().replace(&old_sup, &new_sup);
                *block.text_mut() = updated;
            }
        }
    }

    pub fn toggle_block_type(&mut self, idx: usize) {
        if idx < self.blocks.len() {
            match &self.blocks[idx] {
                MlaBlock::Paragraph {
                    id,
                    text,
                    note_tags,
                } => {
                    self.blocks[idx] = MlaBlock::BlockQuote {
                        id: id.clone(),
                        text: text.clone(),
                        citation: String::new(),
                        note_tags: note_tags.clone(),
                    };
                }
                MlaBlock::BlockQuote {
                    id,
                    text,
                    citation,
                    note_tags,
                } => {
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

        self.reindex_notes();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteShortcutResult {
    pub note_index: usize,
    pub word: String,
}

/// Parses shorthand note creation e.g. `example^1` or `example^13` (or `example^(1)`) followed by a space.
/// When user hits spacebar to confirm the note, `^1` is removed without deleting the associated word,
/// and a NoteTag is added to the block's `note_tags`.
pub fn try_parse_and_apply_note_shortcut(
    text: &mut String,
    note_tags: &mut Vec<NoteTag>,
    existing_note_indices: &[usize],
) -> Result<Option<NoteShortcutResult>, String> {
    let mut search_start = 0;
    while let Some(rel_open) = text[search_start..].find('^') {
        let open_idx = search_start + rel_open;
        let after_caret = &text[open_idx + 1..];

        let (note_num, replace_end) = if after_caret.starts_with('(') {
            // Case 1: ^(digits)
            if let Some(close_rel) = after_caret.find(')') {
                let digits_str = &after_caret[1..close_rel];
                if let Ok(num) = digits_str.parse::<usize>() {
                    if num > 0 {
                        let after_close = &after_caret[close_rel + 1..];
                        // Strictly wait for spacebar confirmation
                        if after_close.starts_with(' ') || after_close.starts_with('\t') {
                            let end = open_idx + 1 + close_rel + 2;
                            (num, end)
                        } else {
                            search_start = open_idx + 1;
                            continue;
                        }
                    } else {
                        search_start = open_idx + 1;
                        continue;
                    }
                } else {
                    search_start = open_idx + 1;
                    continue;
                }
            } else {
                search_start = open_idx + 1;
                continue;
            }
        } else {
            // Case 2: ^digits (e.g. ^1, ^10, ^13)
            let digit_len = after_caret
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .count();
            if digit_len > 0 {
                let digits_str = &after_caret[..digit_len];
                if let Ok(num) = digits_str.parse::<usize>() {
                    if num > 0 {
                        let after_digits = &after_caret[digit_len..];
                        // Strictly wait for spacebar confirmation so typing e.g. 10 does not trigger on 1
                        if after_digits.starts_with(' ') || after_digits.starts_with('\t') {
                            let end = open_idx + 1 + digit_len + 1;
                            (num, end)
                        } else {
                            search_start = open_idx + 1;
                            continue;
                        }
                    } else {
                        search_start = open_idx + 1;
                        continue;
                    }
                } else {
                    search_start = open_idx + 1;
                    continue;
                }
            } else {
                search_start = open_idx + 1;
                continue;
            }
        };

        // Extract preceding word
        let preceding_text = &text[..open_idx];
        let word = preceding_text
            .split_whitespace()
            .last()
            .unwrap_or("")
            .trim_matches(|c: char| {
                c == '"'
                    || c == '\''
                    || c == '('
                    || c == '['
                    || c == '“'
                    || c == '”'
                    || c == ','
                    || c == '.'
                    || c == ';'
                    || c == ':'
            })
            .to_string();

        // In MLA 9, reusing note numbers is strictly prohibited: every note callout must have a unique sequential number.
        if existing_note_indices.contains(&note_num) {
            // Cancel making the note: remove ^N / ^(N) and replace with a normal space
            text.replace_range(open_idx..replace_end, " ");
            return Err(format!(
                "MLA 9 prohibits reusing note numbers. Note {} is already in use; each note must be numbered consecutively.",
                note_num
            ));
        }

        let sup_char = num_to_superscript(note_num);
        let replacement = format!("{} ", sup_char);
        text.replace_range(open_idx..replace_end, &replacement);
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

        return Ok(Some(NoteShortcutResult {
            note_index: note_num,
            word,
        }));
    }

    Ok(None)
}

/// Renders block text by embedding superscript representations right after the tagged words,
/// falling back to appending block notes to the end if no inline tags exist.
pub fn render_text_with_note_tags(
    text: &str,
    tags: &[NoteTag],
    fallback_notes: &[&ExplanatoryNote],
    format_sup: impl Fn(usize) -> String,
) -> String {
    // Strip any raw unicode superscripts so format_sup does not duplicate them
    let base_text = clean_superscripts(text);
    if tags.is_empty() {
        if fallback_notes.is_empty() {
            return base_text;
        } else {
            let sups: String = fallback_notes.iter().map(|n| format_sup(n.index)).collect();
            return format!("{}{}", base_text, sups);
        }
    }

    let mut sorted_tags = tags.to_vec();
    // Sort descending by offset so that insertions do not shift earlier character offsets
    sorted_tags.sort_by_key(|a| std::cmp::Reverse(a.offset));

    let mut result = base_text;
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
        "a", "an", "the", "and", "but", "or", "nor", "for", "so", "yet", "as", "at", "by", "from",
        "in", "into", "of", "off", "on", "onto", "out", "over", "to", "up", "with", "vs", "via",
        "than", "is", "if", "it", "its", "are", "be", "am", "was", "were", "per", "en",
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
                Some(c) if c.is_whitespace() || c == '(' || c == '[' || c == '{' || c == '—' => {
                    true
                }
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
            let next = if i + 1 < len {
                Some(chars[i + 1])
            } else {
                None
            };

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
                Some(c) if c.is_whitespace() || c == '(' || c == '[' || c == '{' || c == '—' => {
                    true
                }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SentenceSpan {
    pub char_start: usize,
    pub char_end: usize,
    pub text: String,
    pub trailing_sep: String,
}

fn is_abbreviation(word: &str) -> bool {
    let lower = word.to_lowercase();
    matches!(
        lower.as_str(),
        "dr" | "mr"
            | "mrs"
            | "ms"
            | "prof"
            | "sr"
            | "jr"
            | "vs"
            | "etc"
            | "eg"
            | "ie"
            | "al"
            | "vol"
            | "no"
            | "ed"
            | "dept"
            | "fig"
            | "co"
            | "inc"
            | "corp"
            | "st"
            | "gen"
            | "gov"
            | "rev"
            | "approx"
            | "avg"
            | "est"
            | "min"
            | "max"
            | "misc"
            | "stat"
            | "univ"
            | "p"
            | "pp"
            | "cf"
            | "ibid"
            | "op"
            | "cit"
    )
}

fn is_terminal_punct(c: char) -> bool {
    c == '.' || c == '!' || c == '?' || c == '‽'
}

fn is_closing_delimiter(c: char) -> bool {
    matches!(
        c,
        '"' | '\''
            | '”'
            | '“'
            | '’'
            | '‘'
            | '»'
            | '«'
            | ')'
            | ']'
            | '}'
            | '⁰'
            | '¹'
            | '²'
            | '³'
            | '⁴'
            | '⁵'
            | '⁶'
            | '⁷'
            | '⁸'
            | '⁹'
    )
}

pub fn split_sentences(text: &str) -> (String, Vec<SentenceSpan>) {
    let chars: Vec<char> = text.chars().collect();
    let total_len = chars.len();

    // 1. Extract leading whitespace
    let mut leading_end = 0;
    while leading_end < total_len && chars[leading_end].is_whitespace() {
        leading_end += 1;
    }
    let leading_ws: String = chars[..leading_end].iter().collect();

    if leading_end == total_len {
        return (leading_ws, Vec::new());
    }

    let mut spans = Vec::new();
    let mut curr_start = leading_end;
    let mut i = leading_end;

    while i < total_len {
        let c = chars[i];
        if is_terminal_punct(c) {
            let mut is_boundary = true;

            // Dot specific exceptions
            if c == '.' {
                // Check decimals (e.g. 3.14) or ellipsis (e.g. ... or ..)
                if (i > 0
                    && chars[i - 1].is_ascii_digit()
                    && i + 1 < total_len
                    && chars[i + 1].is_ascii_digit())
                    || (i > 0 && chars[i - 1] == '.')
                    || (i + 1 < total_len && chars[i + 1] == '.')
                {
                    is_boundary = false;
                }
                // Check abbreviations & single-letter initials
                else {
                    let mut w_start = i;
                    while w_start > curr_start && chars[w_start - 1].is_alphabetic() {
                        w_start -= 1;
                    }
                    if w_start < i {
                        let word: String = chars[w_start..i].iter().collect();
                        if is_abbreviation(&word) {
                            is_boundary = false;
                        }
                    }
                }
            }

            if is_boundary {
                // Consume consecutive terminal punctuation or closing quotes/brackets/superscripts
                let mut end = i + 1;
                while end < total_len
                    && (is_terminal_punct(chars[end]) || is_closing_delimiter(chars[end]))
                {
                    end += 1;
                }

                // Sentence break requires whitespace after the sentence, or reaching the end of text
                if end == total_len || chars[end].is_whitespace() {
                    let s_text: String = chars[curr_start..end].iter().collect();
                    let sep_start = end;
                    while end < total_len && chars[end].is_whitespace() {
                        end += 1;
                    }
                    let sep: String = chars[sep_start..end].iter().collect();

                    spans.push(SentenceSpan {
                        char_start: curr_start,
                        char_end: sep_start,
                        text: s_text,
                        trailing_sep: sep,
                    });

                    curr_start = end;
                    i = end;
                    continue;
                }
            }
        }
        i += 1;
    }

    // Capture any remaining sentence text (e.g., if paragraph doesn't end in punctuation)
    if curr_start < total_len {
        let remainder: String = chars[curr_start..].iter().collect();
        let trimmed = remainder.trim_end();
        let trailing = &remainder[trimmed.len()..];
        let sep_start = curr_start + trimmed.chars().count();
        spans.push(SentenceSpan {
            char_start: curr_start,
            char_end: sep_start,
            text: trimmed.to_string(),
            trailing_sep: trailing.to_string(),
        });
    }

    (leading_ws, spans)
}

pub fn reorder_sentences(
    text: &str,
    cursor_char_idx: usize,
    direction_left: bool,
) -> Option<(String, usize)> {
    let (leading_ws, spans) = split_sentences(text);
    if spans.len() <= 1 {
        return None;
    }

    // Find active sentence index and relative offset within its text
    let mut active_idx = None;
    let mut active_offset = 0;

    for (idx, span) in spans.iter().enumerate() {
        let span_end = span.char_end + span.trailing_sep.chars().count();
        let is_last = idx + 1 == spans.len();

        if cursor_char_idx >= span.char_start && (cursor_char_idx < span_end || is_last) {
            active_idx = Some(idx);
            let s_text_len = span.text.chars().count();
            active_offset = cursor_char_idx
                .saturating_sub(span.char_start)
                .min(s_text_len);
            break;
        }
    }

    let curr_idx = active_idx.unwrap_or(spans.len() - 1);

    let target_idx = if direction_left {
        if curr_idx == 0 {
            return None; // Already at first sentence
        }
        curr_idx - 1
    } else {
        if curr_idx + 1 >= spans.len() {
            return None; // Already at last sentence
        }
        curr_idx + 1
    };

    // Swap texts
    let mut texts: Vec<String> = spans.iter().map(|s| s.text.clone()).collect();
    texts.swap(curr_idx, target_idx);

    // Keep separators in place
    let seps: Vec<String> = spans.iter().map(|s| s.trailing_sep.clone()).collect();

    let mut new_text = leading_ws.clone();
    let mut pos = leading_ws.chars().count();
    let mut new_cursor = 0;

    for k in 0..texts.len() {
        if k == target_idx {
            new_cursor = pos + active_offset;
        }
        new_text.push_str(&texts[k]);
        pos += texts[k].chars().count();
        new_text.push_str(&seps[k]);
        pos += seps[k].chars().count();
    }

    Some((new_text, new_cursor))
}
