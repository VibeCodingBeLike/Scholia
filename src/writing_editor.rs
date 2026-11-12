use crate::model::{MlaBlock, MlaDocument};
use harper_core::linting::{LintGroup, LintKind, Linter, Suggestion};
use harper_core::parsers::PlainEnglish;
use harper_core::spell::{Dictionary, FstDictionary, MergedDictionary, MutableDictionary};
use harper_core::{Dialect, DictWordMetadata, Document};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashSet};
use std::sync::Arc;

/// Supported English regional language variants for the Writing Editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum EnglishLanguage {
    #[default]
    EnglishUS,
    EnglishCA,
    EnglishUK,
}

impl EnglishLanguage {
    pub fn all() -> &'static [EnglishLanguage] {
        &[
            EnglishLanguage::EnglishUS,
            EnglishLanguage::EnglishCA,
            EnglishLanguage::EnglishUK,
        ]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            EnglishLanguage::EnglishUS => "English (United States) — en-US",
            EnglishLanguage::EnglishCA => "English (Canada) — en-CA",
            EnglishLanguage::EnglishUK => "English (United Kingdom) — en-UK",
        }
    }

    pub fn short_code(&self) -> &'static str {
        match self {
            EnglishLanguage::EnglishUS => "en-US",
            EnglishLanguage::EnglishCA => "en-CA",
            EnglishLanguage::EnglishUK => "en-UK",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            EnglishLanguage::EnglishUS => {
                "Standard American English spelling (e.g., color, analyze, center, defense)."
            }
            EnglishLanguage::EnglishCA => {
                "Canadian English — allows both US and UK spelling variants (e.g., color/colour, analyze/analyse, center/centre)."
            }
            EnglishLanguage::EnglishUK => {
                "Standard British English spelling (e.g., colour, analyse, centre, defence)."
            }
        }
    }

    pub fn to_harper_dialect(self) -> Dialect {
        match self {
            EnglishLanguage::EnglishUS => Dialect::American,
            EnglishLanguage::EnglishCA => Dialect::Canadian,
            EnglishLanguage::EnglishUK => Dialect::British,
        }
    }
}

fn default_true() -> bool {
    true
}

/// User-configurable settings for the Writing Editor (persisted in mla_config.json).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EditorSettings {
    #[serde(default)]
    pub language: EnglishLanguage,
    #[serde(default = "default_true")]
    pub enable_spell_check: bool,
    #[serde(default = "default_true")]
    pub enable_grammar_check: bool,
    #[serde(default = "default_true")]
    pub enable_style_check: bool,
    #[serde(default = "default_true")]
    pub flag_contractions: bool,
    #[serde(default = "default_true")]
    pub ignore_all_caps: bool,
    #[serde(default = "default_true")]
    pub ignore_words_with_numbers: bool,
    #[serde(default)]
    pub custom_dictionary: BTreeSet<String>,
    #[serde(skip)]
    pub ignored_issues: HashSet<String>,
    #[serde(skip)]
    pub ignored_words_session: HashSet<String>,
}

impl Default for EditorSettings {
    fn default() -> Self {
        Self {
            language: EnglishLanguage::EnglishUS,
            enable_spell_check: true,
            enable_grammar_check: true,
            enable_style_check: true,
            flag_contractions: true,
            ignore_all_caps: true,
            ignore_words_with_numbers: true,
            custom_dictionary: BTreeSet::new(),
            ignored_issues: HashSet::new(),
            ignored_words_session: HashSet::new(),
        }
    }
}

impl EditorSettings {
    /// Normalize a word for dictionary storage/lookup.
    pub fn normalize_word(word: &str) -> String {
        word.trim()
            .trim_matches(|c: char| !c.is_alphabetic() && c != '\'' && c != '’' && c != '-')
            .replace('’', "'")
            .to_lowercase()
    }

    /// Add a word to the persistent custom dictionary. Returns true if newly inserted.
    pub fn add_to_dictionary(&mut self, word: &str) -> bool {
        let norm = Self::normalize_word(word);
        if norm.is_empty() {
            return false;
        }
        self.custom_dictionary.insert(norm)
    }

    /// Remove a word from the persistent custom dictionary.
    pub fn remove_from_dictionary(&mut self, word: &str) -> bool {
        let norm = Self::normalize_word(word);
        self.custom_dictionary.remove(&norm)
    }

    /// Check if a word is present in the persistent custom dictionary or session-ignored words.
    pub fn is_word_accepted(&self, word: &str) -> bool {
        let norm = Self::normalize_word(word);
        if norm.is_empty() {
            return true;
        }
        self.custom_dictionary.contains(&norm) || self.ignored_words_session.contains(&norm)
    }

    /// Ignore a specific suggestion occurrence for the current session.
    pub fn ignore_issue(&mut self, issue_key: &str) {
        self.ignored_issues.insert(issue_key.to_string());
    }

    /// Ignore all occurrences of a word for the current session.
    pub fn ignore_word_for_session(&mut self, word: &str) {
        let norm = Self::normalize_word(word);
        if !norm.is_empty() {
            self.ignored_words_session.insert(norm);
        }
    }

    /// Clear all session-ignored suggestions and words.
    pub fn clear_ignored(&mut self) {
        self.ignored_issues.clear();
        self.ignored_words_session.clear();
    }
}

/// High-level category for a Writing Editor suggestion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SuggestionCategory {
    Spelling,
    Grammar,
    Punctuation,
    Style,
}

impl SuggestionCategory {
    pub fn display_name(&self) -> &'static str {
        match self {
            SuggestionCategory::Spelling => "Spelling",
            SuggestionCategory::Grammar => "Grammar",
            SuggestionCategory::Punctuation => "Punctuation",
            SuggestionCategory::Style => "Clarity & Style",
        }
    }
}

/// Which part of the MLA document an issue belongs to.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EditorTarget {
    Title,
    Block(usize),
    Note(usize),
}

/// Concrete replacement operation when the user accepts a suggestion.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReplacementAction {
    Replace(String),
    InsertAfter(String),
    Remove,
}

impl ReplacementAction {
    pub fn button_label(&self) -> String {
        match self {
            ReplacementAction::Replace(s) => format!("✓ \"{}\"", s),
            ReplacementAction::InsertAfter(s) => format!("✓ Insert \"{}\"", s),
            ReplacementAction::Remove => "✓ Remove".to_string(),
        }
    }
}

/// A single actionable issue surfaced by the Writing Editor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorIssue {
    pub issue_key: String,
    pub target: EditorTarget,
    pub location_label: String,
    pub category: SuggestionCategory,
    pub kind_label: String,
    pub char_start: usize,
    pub char_end: usize,
    pub flagged_text: String,
    pub context_before: String,
    pub context_after: String,
    pub message: String,
    pub suggestions: Vec<ReplacementAction>,
    pub can_add_to_dictionary: bool,
}

/// Full report returned by the Writing Editor, including document readability insights.
#[derive(Debug, Clone, Default)]
pub struct EditorReport {
    pub issues: Vec<EditorIssue>,
    pub spelling_count: usize,
    pub grammar_count: usize,
    pub punctuation_count: usize,
    pub style_count: usize,
    pub editor_score: u32,
    pub flesch_reading_ease: f32,
    pub flesch_kincaid_grade: f32,
    pub avg_sentence_length: f32,
    pub vocabulary_diversity_pct: f32,
}

impl EditorReport {
    pub fn total_suggestions(&self) -> usize {
        self.issues.len()
    }
}

/// Prewarm the curated dictionary in a background thread so the first check has zero latency.
pub fn prewarm_editor() {
    use std::sync::atomic::{AtomicBool, Ordering};
    static STARTED: AtomicBool = AtomicBool::new(false);
    if STARTED.swap(true, Ordering::AcqRel) {
        return;
    }
    std::thread::spawn(|| {
        let _ = FstDictionary::curated();
    });
}

pub struct WritingEditor;

impl WritingEditor {
    /// Inspect the entire MLA document and return an `EditorReport`.
    pub fn inspect(doc: &MlaDocument, settings: &EditorSettings) -> EditorReport {
        let mut issues = Vec::new();

        // Build a merged dictionary with FstDictionary::curated() + user's custom dictionary + document proper nouns
        let base_dict = FstDictionary::curated();
        let mut user_dict = MutableDictionary::new();
        let default_meta = DictWordMetadata {
            common: true,
            ..Default::default()
        };

        for word in &settings.custom_dictionary {
            if !word.is_empty() {
                user_dict.append_word_str(word, default_meta.clone());
                // Also add capitalized version so sentence-start or proper-noun usage is recognized
                let cap = capitalize_first(word);
                if cap != *word {
                    user_dict.append_word_str(&cap, default_meta.clone());
                }
            }
        }
        for word in &settings.ignored_words_session {
            if !word.is_empty() {
                user_dict.append_word_str(word, default_meta.clone());
                let cap = capitalize_first(word);
                if cap != *word {
                    user_dict.append_word_str(&cap, default_meta.clone());
                }
            }
        }

        // Automatically recognize student name, instructor name, and Works Cited author names
        for name_token in extract_document_proper_nouns(doc) {
            user_dict.append_word_str(&name_token, default_meta.clone());
            user_dict.append_word_str(&name_token.to_lowercase(), default_meta.clone());
        }

        let mut merged_dict = MergedDictionary::new();
        merged_dict.add_dictionary(base_dict.clone());
        merged_dict.add_dictionary(Arc::new(user_dict));
        let merged_arc = Arc::new(merged_dict);

        // 1. Check Title
        if !doc.title.trim().is_empty() {
            Self::check_text_segment(
                &doc.title,
                EditorTarget::Title,
                "Paper Title",
                settings,
                &base_dict,
                &merged_arc,
                &mut issues,
            );
        }

        // 2. Check Body Blocks
        let mut p_num = 0usize;
        let mut bq_num = 0usize;
        let mut h_num = 0usize;
        for (idx, block) in doc.blocks.iter().enumerate() {
            let label = match block {
                MlaBlock::Paragraph { .. } => {
                    p_num += 1;
                    format!("Paragraph {}", p_num)
                }
                MlaBlock::BlockQuote { .. } => {
                    bq_num += 1;
                    format!("Block Quote {}", bq_num)
                }
                MlaBlock::SectionHeading { level, .. } => {
                    h_num += 1;
                    format!("Heading {} (L{})", h_num, level)
                }
            };
            let text = block.text();
            if !text.trim().is_empty() {
                Self::check_text_segment(
                    text,
                    EditorTarget::Block(idx),
                    &label,
                    settings,
                    &base_dict,
                    &merged_arc,
                    &mut issues,
                );
            }
        }

        // 3. Check Explanatory Notes
        for (idx, note) in doc.notes.iter().enumerate() {
            if !note.text.trim().is_empty() {
                let label = format!("Footnote {}", note.index);
                Self::check_text_segment(
                    &note.text,
                    EditorTarget::Note(idx),
                    &label,
                    settings,
                    &base_dict,
                    &merged_arc,
                    &mut issues,
                );
            }
        }

        // Count categories
        let mut spelling_count = 0;
        let mut grammar_count = 0;
        let mut punctuation_count = 0;
        let mut style_count = 0;
        for issue in &issues {
            match issue.category {
                SuggestionCategory::Spelling => spelling_count += 1,
                SuggestionCategory::Grammar => grammar_count += 1,
                SuggestionCategory::Punctuation => punctuation_count += 1,
                SuggestionCategory::Style => style_count += 1,
            }
        }

        // Calculate readability & Editor score
        let (flesch_ease, fk_grade, avg_sent_len, vocab_div) = compute_readability_metrics(doc);
        let penalty = (spelling_count * 5)
            + (grammar_count * 5)
            + (punctuation_count * 3)
            + (style_count * 2);
        let editor_score = if penalty >= 100 {
            0
        } else {
            100 - penalty as u32
        };

        EditorReport {
            issues,
            spelling_count,
            grammar_count,
            punctuation_count,
            style_count,
            editor_score,
            flesch_reading_ease: flesch_ease,
            flesch_kincaid_grade: fk_grade,
            avg_sentence_length: avg_sent_len,
            vocabulary_diversity_pct: vocab_div,
        }
    }

    fn check_text_segment(
        text: &str,
        target: EditorTarget,
        location_label: &str,
        settings: &EditorSettings,
        base_dict: &Arc<FstDictionary>,
        merged_dict: &Arc<MergedDictionary>,
        out_issues: &mut Vec<EditorIssue>,
    ) {
        if !settings.enable_spell_check
            && !settings.enable_grammar_check
            && !settings.enable_style_check
            && !settings.flag_contractions
        {
            return;
        }

        let chars: Vec<char> = text.chars().collect();
        // Replace footnote superscripts (¹²³ etc.) with spaces of identical char length so Harper doesn't attach them to words
        let sanitized: String = chars
            .iter()
            .map(|&c| if is_superscript_char(c) { ' ' } else { c })
            .collect();

        let dialect = settings.language.to_harper_dialect();
        let harper_doc = Document::new(&sanitized, &PlainEnglish, merged_dict);
        let mut linter = LintGroup::new_curated(merged_dict.clone(), dialect);
        let mut lints = linter.lint(&harper_doc);
        harper_core::remove_overlaps(&mut lints);

        let mut covered_ranges: Vec<(usize, usize)> = Vec::new();

        for lint in lints {
            let start = lint.span.start;
            let end = lint.span.end;
            if start >= end || end > chars.len() {
                continue;
            }

            let flagged_text: String = chars[start..end].iter().collect();
            if flagged_text.trim().is_empty() {
                continue;
            }

            // Map Harper LintKind to our SuggestionCategory
            let (category, kind_label) = categorize_lint_kind(lint.lint_kind);

            // Check category toggles
            match category {
                SuggestionCategory::Spelling if !settings.enable_spell_check => continue,
                SuggestionCategory::Grammar | SuggestionCategory::Punctuation
                    if !settings.enable_grammar_check =>
                {
                    continue;
                }
                SuggestionCategory::Style if !settings.enable_style_check => continue,
                _ => {}
            }

            // Ignore words in ALL CAPS if configured
            if settings.ignore_all_caps
                && category == SuggestionCategory::Spelling
                && flagged_text.chars().filter(|c| c.is_alphabetic()).count() >= 2
                && flagged_text
                    .chars()
                    .filter(|c| c.is_alphabetic())
                    .all(|c| c.is_uppercase())
            {
                continue;
            }

            // Ignore words with numbers if configured
            if settings.ignore_words_with_numbers
                && category == SuggestionCategory::Spelling
                && flagged_text.chars().any(|c| c.is_ascii_digit())
            {
                continue;
            }

            // Skip custom dictionary / session-ignored words
            if category == SuggestionCategory::Spelling && settings.is_word_accepted(&flagged_text)
            {
                continue;
            }

            // Special handling for Canadian English (en-CA):
            // Allow both US and UK spellings without flagging them as errors!
            if settings.language == EnglishLanguage::EnglishCA
                && (lint.lint_kind == LintKind::Spelling || lint.lint_kind == LintKind::Regionalism)
                && is_valid_in_us_uk_or_ca(&flagged_text, base_dict)
            {
                continue;
            }

            let issue_key = format!(
                "{:?}:{}:{}:{}:{:?}",
                target, start, end, flagged_text, lint.lint_kind
            );
            if settings.ignored_issues.contains(&issue_key) {
                continue;
            }

            let suggestions: Vec<ReplacementAction> = lint
                .suggestions
                .into_iter()
                .filter_map(|sug| match sug {
                    Suggestion::ReplaceWith(rep_chars) => {
                        let rep: String = rep_chars.into_iter().collect();
                        if rep != flagged_text {
                            Some(ReplacementAction::Replace(rep))
                        } else {
                            None
                        }
                    }
                    Suggestion::InsertAfter(ins_chars) => {
                        let ins: String = ins_chars.into_iter().collect();
                        if !ins.is_empty() {
                            Some(ReplacementAction::InsertAfter(ins))
                        } else {
                            None
                        }
                    }
                    Suggestion::Remove => Some(ReplacementAction::Remove),
                })
                .take(4)
                .collect();

            let (context_before, context_after) = extract_context_window(&chars, start, end);
            let is_single_word = !flagged_text.trim().contains(char::is_whitespace);

            covered_ranges.push((start, end));
            out_issues.push(EditorIssue {
                issue_key,
                target: target.clone(),
                location_label: location_label.to_string(),
                category,
                kind_label,
                char_start: start,
                char_end: end,
                flagged_text,
                context_before,
                context_after,
                message: clean_markdown_backticks(&lint.message),
                suggestions,
                can_add_to_dictionary: category == SuggestionCategory::Spelling && is_single_word,
            });
        }

        // Additional Academic MLA Formal Contraction & Style Checks (if not already covered by a Harper span)
        if settings.flag_contractions && !matches!(target, EditorTarget::Title) {
            Self::check_academic_contractions(
                &chars,
                &target,
                location_label,
                settings,
                &covered_ranges,
                out_issues,
            );
        }
    }

    fn check_academic_contractions(
        chars: &[char],
        target: &EditorTarget,
        location_label: &str,
        settings: &EditorSettings,
        covered_ranges: &[(usize, usize)],
        out_issues: &mut Vec<EditorIssue>,
    ) {
        const CONTRACTIONS: &[(&str, &str)] = &[
            ("don't", "do not"),
            ("doesn't", "does not"),
            ("didn't", "did not"),
            ("can't", "cannot"),
            ("couldn't", "could not"),
            ("won't", "will not"),
            ("wouldn't", "would not"),
            ("shouldn't", "should not"),
            ("isn't", "is not"),
            ("aren't", "are not"),
            ("wasn't", "was not"),
            ("weren't", "were not"),
            ("hasn't", "has not"),
            ("haven't", "have not"),
            ("hadn't", "had not"),
            ("you're", "you are"),
            ("they're", "they are"),
            ("we're", "we are"),
            ("I'm", "I am"),
            ("it's", "it is"),
            ("that's", "that is"),
            ("there's", "there is"),
        ];

        // Scan words with apostrophes
        let len = chars.len();
        let mut i = 0;
        while i < len {
            if !chars[i].is_alphabetic() {
                i += 1;
                continue;
            }
            let start = i;
            while i < len && (chars[i].is_alphabetic() || chars[i] == '\'' || chars[i] == '’') {
                i += 1;
            }
            let end = i;
            if covered_ranges
                .iter()
                .any(|&(cs, ce)| start < ce && end > cs)
            {
                continue;
            }

            let raw_word: String = chars[start..end].iter().collect();
            let norm_word = raw_word.replace('’', "'");
            if settings.is_word_accepted(&norm_word) {
                continue;
            }

            let lower = norm_word.to_lowercase();
            if let Some(&(_, expanded)) = CONTRACTIONS
                .iter()
                .find(|&&(c, _)| c.eq_ignore_ascii_case(&lower))
            {
                let replacement = if chars[start].is_uppercase() && expanded != "I am" {
                    capitalize_first(expanded)
                } else {
                    expanded.to_string()
                };

                let issue_key = format!("{:?}:{}:{}:{}:Contraction", target, start, end, raw_word);
                if settings.ignored_issues.contains(&issue_key) {
                    continue;
                }

                let (context_before, context_after) = extract_context_window(chars, start, end);
                out_issues.push(EditorIssue {
                    issue_key,
                    target: target.clone(),
                    location_label: location_label.to_string(),
                    category: SuggestionCategory::Style,
                    kind_label: "Academic Tone".to_string(),
                    char_start: start,
                    char_end: end,
                    flagged_text: raw_word.clone(),
                    context_before,
                    context_after,
                    message: format!(
                        "Avoid contractions like \"{}\" in formal academic MLA prose; prefer \"{}\".",
                        raw_word, replacement
                    ),
                    suggestions: vec![ReplacementAction::Replace(replacement)],
                    can_add_to_dictionary: false,
                });
            }
        }
    }

    /// Apply a chosen `ReplacementAction` for a single `EditorIssue` to `doc`.
    /// Returns `true` if the document was modified.
    pub fn apply_suggestion(
        doc: &mut MlaDocument,
        issue: &EditorIssue,
        action: &ReplacementAction,
    ) -> bool {
        let modified = match &issue.target {
            EditorTarget::Title => apply_to_string(
                &mut doc.title,
                issue.char_start,
                issue.char_end,
                &issue.flagged_text,
                action,
            ),
            EditorTarget::Block(idx) => {
                if let Some(block) = doc.blocks.get_mut(*idx) {
                    apply_to_string(
                        block.text_mut(),
                        issue.char_start,
                        issue.char_end,
                        &issue.flagged_text,
                        action,
                    )
                } else {
                    false
                }
            }
            EditorTarget::Note(idx) => {
                if let Some(note) = doc.notes.get_mut(*idx) {
                    apply_to_string(
                        &mut note.text,
                        issue.char_start,
                        issue.char_end,
                        &issue.flagged_text,
                        action,
                    )
                } else {
                    false
                }
            }
        };

        if modified {
            doc.sync_body_from_blocks();
            doc.is_dirty = true;
        }
        modified
    }

    /// Apply the top suggestion for all actionable issues in reverse character order per target.
    /// Returns the number of suggestions applied.
    pub fn apply_all_top_suggestions(doc: &mut MlaDocument, settings: &EditorSettings) -> usize {
        let report = Self::inspect(doc, settings);
        // Filter issues that have at least one suggestion, and sort in reverse order of (char_start)
        let mut actionable: Vec<&EditorIssue> = report
            .issues
            .iter()
            .filter(|i| !i.suggestions.is_empty())
            .collect();

        // Sort reverse by char_start so earlier edits in the same field never invalidate offsets
        actionable.sort_by_key(|a| std::cmp::Reverse(a.char_start));

        let mut applied_count = 0;
        for issue in actionable {
            if let Some(first_action) = issue.suggestions.first() {
                if Self::apply_suggestion(doc, issue, first_action) {
                    applied_count += 1;
                }
            }
        }
        applied_count
    }
}

fn apply_to_string(
    target_str: &mut String,
    char_start: usize,
    char_end: usize,
    expected_flagged: &str,
    action: &ReplacementAction,
) -> bool {
    let mut chars: Vec<char> = target_str.chars().collect();
    if char_start <= char_end && char_end <= chars.len() {
        let current_slice: String = chars[char_start..char_end].iter().collect();
        if current_slice == expected_flagged {
            match action {
                ReplacementAction::Replace(rep) => {
                    chars.splice(char_start..char_end, rep.chars());
                }
                ReplacementAction::InsertAfter(ins) => {
                    chars.splice(char_end..char_end, ins.chars());
                }
                ReplacementAction::Remove => {
                    chars.drain(char_start..char_end);
                }
            }
            *target_str = chars.into_iter().collect();
            return true;
        }
    }

    // Fallback if earlier edits shifted character indices: find exact match of expected_flagged
    if !expected_flagged.is_empty() {
        if let Some(byte_pos) = target_str.find(expected_flagged) {
            let end_byte = byte_pos + expected_flagged.len();
            match action {
                ReplacementAction::Replace(rep) => {
                    target_str.replace_range(byte_pos..end_byte, rep);
                }
                ReplacementAction::InsertAfter(ins) => {
                    target_str.insert_str(end_byte, ins);
                }
                ReplacementAction::Remove => {
                    target_str.replace_range(byte_pos..end_byte, "");
                }
            }
            return true;
        }
    }
    false
}

/// Checks whether `word` is valid in US, UK, or Canadian English in the curated dictionary.
fn is_valid_in_us_uk_or_ca(word: &str, dict: &Arc<FstDictionary>) -> bool {
    let norm = word.replace('’', "'");
    let chars: Vec<char> = norm.chars().collect();
    let lower_chars: Vec<char> = norm.to_lowercase().chars().collect();

    let Some(meta) = dict.get_word_metadata(&chars) else {
        return false;
    };

    let dialect_ok = meta.dialects.is_dialect_enabled(Dialect::Canadian)
        || meta.dialects.is_dialect_enabled(Dialect::American)
        || meta.dialects.is_dialect_enabled(Dialect::British);

    let exact_ok = dict.contains_exact_word(&chars) || dict.contains_exact_word(&lower_chars);
    dialect_ok && exact_ok
}

fn categorize_lint_kind(kind: LintKind) -> (SuggestionCategory, String) {
    match kind {
        LintKind::Spelling | LintKind::Typo | LintKind::Regionalism => {
            (SuggestionCategory::Spelling, kind.to_string())
        }
        LintKind::Punctuation | LintKind::Formatting => {
            (SuggestionCategory::Punctuation, kind.to_string())
        }
        LintKind::Style
        | LintKind::Readability
        | LintKind::Redundancy
        | LintKind::Enhancement
        | LintKind::WordChoice => (SuggestionCategory::Style, kind.to_string()),
        _ => (SuggestionCategory::Grammar, kind.to_string()),
    }
}

fn extract_context_window(chars: &[char], start: usize, end: usize) -> (String, String) {
    let before_start = start.saturating_sub(28);
    let after_end = (end + 28).min(chars.len());

    let mut before: String = chars[before_start..start].iter().collect();
    if before_start > 0 {
        before = format!("…{}", before.trim_start());
    }

    let mut after: String = chars[end..after_end].iter().collect();
    if after_end < chars.len() {
        after = format!("{}…", after.trim_end());
    }

    (before, after)
}

fn clean_markdown_backticks(msg: &str) -> String {
    msg.replace('`', "\"")
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

fn is_superscript_char(c: char) -> bool {
    matches!(c, '⁰' | '¹' | '²' | '³' | '⁴' | '⁵' | '⁶' | '⁷' | '⁸' | '⁹')
}

fn extract_document_proper_nouns(doc: &MlaDocument) -> Vec<String> {
    let mut nouns = Vec::new();
    for field in [
        doc.header.student_name.as_str(),
        doc.header.instructor_name.as_str(),
        doc.header.running_header_last_name.as_str(),
    ] {
        for part in field.split(|c: char| !c.is_alphabetic() && c != '-' && c != '\'') {
            let trimmed = part.trim();
            if trimmed.len() >= 2 {
                nouns.push(trimmed.to_string());
            }
        }
    }
    for entry in &doc.works_cited {
        for part in entry
            .author
            .split(|c: char| !c.is_alphabetic() && c != '-' && c != '\'')
        {
            let trimmed = part.trim();
            if trimmed.len() >= 2 {
                nouns.push(trimmed.to_string());
            }
        }
    }
    nouns
}

fn count_syllables(word: &str) -> usize {
    let lower: Vec<char> = word
        .chars()
        .filter(|c| c.is_ascii_alphabetic())
        .map(|c| c.to_ascii_lowercase())
        .collect();
    if lower.is_empty() {
        return 1;
    }
    if lower.len() <= 3 {
        return 1;
    }

    let is_vowel = |c: char| matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'y');
    let mut count = 0usize;
    let mut prev_vowel = false;

    for &c in &lower {
        let v = is_vowel(c);
        if v && !prev_vowel {
            count += 1;
        }
        prev_vowel = v;
    }

    // Silent trailing 'e'
    if lower.ends_with(&['e']) && !lower.ends_with(&['l', 'e']) && count > 1 {
        count -= 1;
    }

    count.max(1)
}

fn compute_readability_metrics(doc: &MlaDocument) -> (f32, f32, f32, f32) {
    let mut words: Vec<String> = Vec::new();
    let mut sentence_count = 0usize;

    for block in &doc.blocks {
        let text = block.text().trim();
        if text.is_empty() {
            continue;
        }
        let (_, spans) = crate::model::split_sentences(text);
        sentence_count += spans.len().max(1);
        for raw in text.split_whitespace() {
            let clean: String = raw
                .chars()
                .filter(|c| c.is_alphabetic())
                .map(|c| c.to_lowercase().next().unwrap_or(c))
                .collect();
            if !clean.is_empty() {
                words.push(clean);
            }
        }
    }

    if words.is_empty() {
        return (100.0, 0.0, 0.0, 100.0);
    }

    let total_words = words.len() as f32;
    let total_sentences = (sentence_count.max(1)) as f32;
    let total_syllables: usize = words.iter().map(|w| count_syllables(w)).sum();

    let avg_sent_len = total_words / total_sentences;
    let avg_syllables_per_word = (total_syllables as f32) / total_words;

    let flesch_ease =
        (206.835 - (1.015 * avg_sent_len) - (84.6 * avg_syllables_per_word)).clamp(0.0, 100.0);
    let fk_grade =
        ((0.39 * avg_sent_len) + (11.8 * avg_syllables_per_word) - 15.59).clamp(0.0, 22.0);

    let unique_words: HashSet<&String> = words.iter().collect();
    let vocab_div = ((unique_words.len() as f32) / total_words * 100.0).clamp(0.0, 100.0);

    (flesch_ease, fk_grade, avg_sent_len, vocab_div)
}
