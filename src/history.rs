use crate::model::MlaDocument;
use std::time::Instant;

/// Determines if a character triggers a word boundary checkpoint in undo history
pub fn is_word_boundary_char(c: char) -> bool {
    c.is_whitespace()
        || matches!(
            c,
            '.' | ','
                | '!'
                | '?'
                | ';'
                | ':'
                | '—'
                | '-'
                | '"'
                | '\''
                | '”'
                | '“'
                | '’'
                | '‘'
                | '('
                | ')'
                | '['
                | ']'
                | '{'
                | '}'
                | '/'
                | '\\'
                | '^'
        )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    pub doc: MlaDocument,
    pub cursor_pos: usize,
}

#[derive(Debug, Clone)]
pub struct HistoryManager {
    pub undo_stack: Vec<Snapshot>,
    pub redo_stack: Vec<Snapshot>,
    /// Base snapshot before the current typing group began
    pub typing_base: Option<Snapshot>,
    /// Last committed document (to detect changes)
    pub last_doc: MlaDocument,
    /// Last time an edit was detected
    pub last_edit_time: Instant,
    /// Maximum number of undo states
    pub max_depth: usize,
}

impl HistoryManager {
    pub fn new(initial_doc: &MlaDocument) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            typing_base: None,
            last_doc: initial_doc.clone(),
            last_edit_time: Instant::now(),
            max_depth: 512,
        }
    }

    pub fn set_max_depth(&mut self, depth: usize) {
        self.max_depth = depth.clamp(16, 8192);
        while self.undo_stack.len() > self.max_depth {
            self.undo_stack.remove(0);
        }
        while self.redo_stack.len() > self.max_depth {
            self.redo_stack.remove(0);
        }
    }

    pub fn reset(&mut self, new_doc: &MlaDocument) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.typing_base = None;
        self.last_doc = new_doc.clone();
        self.last_edit_time = Instant::now();
    }

    pub fn can_undo(&self) -> bool {
        self.typing_base.is_some() || !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Record a discrete action (e.g. move block, move sentence, delete block, title case, citation).
    /// Pushes the pre-action document to the undo stack and clears the redo stack.
    pub fn record_discrete_action(&mut self, current_doc: &MlaDocument, cursor_pos: usize) {
        if let Some(base) = self.typing_base.take() {
            self.undo_stack.push(base);
        }
        self.undo_stack.push(Snapshot {
            doc: current_doc.clone(),
            cursor_pos,
        });
        if self.undo_stack.len() > self.max_depth {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
        self.last_doc = current_doc.clone();
        self.last_edit_time = Instant::now();
    }

    /// Called at the end of each frame to observe typing changes, group words, and commit when word boundaries or pauses occur.
    pub fn on_frame_end(&mut self, current_doc: &MlaDocument, current_cursor_pos: usize) {
        if current_doc.content_equals(&self.last_doc) {
            // No content changes this frame.
            // Check for typing pause (> 1000ms): commit typing group!
            if self.typing_base.is_some() && self.last_edit_time.elapsed().as_millis() > 1000 {
                let base = self.typing_base.take().unwrap();
                self.undo_stack.push(base);
                if self.undo_stack.len() > self.max_depth {
                    self.undo_stack.remove(0);
                }
                self.redo_stack.clear();
            }
            return;
        }

        // Content changed!
        let is_boundary = self.detect_word_boundary(&self.last_doc, current_doc);

        if self.typing_base.is_none() {
            // Start of a new typing group: save the state BEFORE this typing group began!
            self.typing_base = Some(Snapshot {
                doc: self.last_doc.clone(),
                cursor_pos: current_cursor_pos,
            });
        }

        self.last_edit_time = Instant::now();

        if is_boundary {
            // A word completed: commit the typing group to undo stack!
            let base = self.typing_base.take().unwrap();
            self.undo_stack.push(base);
            if self.undo_stack.len() > self.max_depth {
                self.undo_stack.remove(0);
            }
            self.redo_stack.clear();
        }

        self.last_doc = current_doc.clone();
    }

    /// Perform Undo: roll back to the previous snapshot.
    pub fn undo(
        &mut self,
        current_doc: &MlaDocument,
        current_cursor_pos: usize,
    ) -> Option<Snapshot> {
        // If there's an active uncommitted typing group, roll back to its base!
        if let Some(base) = self.typing_base.take() {
            self.redo_stack.push(Snapshot {
                doc: current_doc.clone(),
                cursor_pos: current_cursor_pos,
            });
            if self.redo_stack.len() > self.max_depth {
                self.redo_stack.remove(0);
            }
            self.last_doc = base.doc.clone();
            self.last_edit_time = Instant::now();
            return Some(base);
        }

        if let Some(prev) = self.undo_stack.pop() {
            self.redo_stack.push(Snapshot {
                doc: current_doc.clone(),
                cursor_pos: current_cursor_pos,
            });
            if self.redo_stack.len() > self.max_depth {
                self.redo_stack.remove(0);
            }
            self.last_doc = prev.doc.clone();
            self.last_edit_time = Instant::now();
            Some(prev)
        } else {
            None
        }
    }

    /// Perform Redo: roll forward to the next snapshot.
    pub fn redo(
        &mut self,
        current_doc: &MlaDocument,
        current_cursor_pos: usize,
    ) -> Option<Snapshot> {
        if let Some(next) = self.redo_stack.pop() {
            // Commit any typing base before redo
            if let Some(base) = self.typing_base.take() {
                self.undo_stack.push(base);
                if self.undo_stack.len() > self.max_depth {
                    self.undo_stack.remove(0);
                }
            }
            self.undo_stack.push(Snapshot {
                doc: current_doc.clone(),
                cursor_pos: current_cursor_pos,
            });
            if self.undo_stack.len() > self.max_depth {
                self.undo_stack.remove(0);
            }
            self.last_doc = next.doc.clone();
            self.last_edit_time = Instant::now();
            Some(next)
        } else {
            None
        }
    }

    fn detect_word_boundary(&self, old_doc: &MlaDocument, new_doc: &MlaDocument) -> bool {
        // Structural changes (block count, notes, works cited, active block)
        if old_doc.blocks.len() != new_doc.blocks.len()
            || old_doc.works_cited.len() != new_doc.works_cited.len()
            || old_doc.notes.len() != new_doc.notes.len()
            || old_doc.active_block_idx != new_doc.active_block_idx
        {
            return true;
        }

        // Active block text changes
        let b_idx = new_doc
            .active_block_idx
            .min(new_doc.blocks.len().saturating_sub(1));
        if let (Some(old_b), Some(new_b)) = (old_doc.blocks.get(b_idx), new_doc.blocks.get(b_idx)) {
            let old_text = old_b.text();
            let new_text = new_b.text();

            if old_text != new_text {
                if new_text.len() > old_text.len() {
                    let diff_len = new_text.len() - old_text.len();
                    if diff_len > 4 {
                        return true; // Large change (paste/replace)
                    }
                    if new_text
                        .chars()
                        .rev()
                        .take(diff_len)
                        .any(is_word_boundary_char)
                    {
                        return true;
                    }
                } else if new_text.len() < old_text.len() {
                    let diff_len = old_text.len() - new_text.len();
                    if diff_len > 4 {
                        return true; // Large deletion
                    }
                    if old_text
                        .chars()
                        .rev()
                        .take(diff_len)
                        .any(is_word_boundary_char)
                    {
                        return true;
                    }
                }
            }
        }

        // Header or title changes
        if old_doc.title != new_doc.title
            && new_doc
                .title
                .chars()
                .last()
                .is_some_and(is_word_boundary_char)
        {
            return true;
        }
        if old_doc.header != new_doc.header
            && (new_doc
                .header
                .student_name
                .chars()
                .last()
                .is_some_and(is_word_boundary_char)
                || new_doc
                    .header
                    .instructor_name
                    .chars()
                    .last()
                    .is_some_and(is_word_boundary_char)
                || new_doc
                    .header
                    .course
                    .chars()
                    .last()
                    .is_some_and(is_word_boundary_char))
        {
            return true;
        }

        false
    }
}
