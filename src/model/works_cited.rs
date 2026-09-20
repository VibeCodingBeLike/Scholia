use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceType {
    /// E.g. Book, Novel, Standalone Website, Film, Play (Title is italicized)
    BookOrStandalone,
    /// E.g. Article, Essay, Short Story, Poem, Web Page, Song (Title in quotes, container italicized)
    ArticleOrChapter,
    /// Scholarly Journal Article (Title in quotes, Journal italicized, vol/no/date/pages)
    JournalArticle,
    /// Online Website Page (Title in quotes, Site Name italicized, URL)
    WebPage,
    /// Custom raw entry if user wants full custom MLA text
    Custom,
}

impl SourceType {
    pub fn display_name(&self) -> &'static str {
        match self {
            SourceType::BookOrStandalone => "Book / Standalone Work",
            SourceType::ArticleOrChapter => "Article / Short Work / Chapter",
            SourceType::JournalArticle => "Scholarly Journal Article",
            SourceType::WebPage => "Web Page / Online Article",
            SourceType::Custom => "Custom Entry",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorksCitedEntry {
    pub id: String,
    pub source_type: SourceType,
    /// E.g. "Morrison, Toni" or "Smith, John, and Jane Doe"
    pub author: String,
    /// E.g. "Beloved" or "The Art of Fiction"
    pub title_of_source: String,
    /// E.g. "The Atlantic" or "Knopf"
    pub container_title: String,
    /// E.g. "edited by John Doe" or "translated by Jane Doe"
    pub other_contributors: String,
    /// E.g. "2nd ed."
    pub version: String,
    /// E.g. "vol. 14, no. 2"
    pub number: String,
    /// E.g. "Penguin Random House"
    pub publisher: String,
    /// E.g. "2021" or "15 May 2023"
    pub pub_date: String,
    /// E.g. "pp. 112-125" or "https://..." or "doi:..."
    pub location: String,
    /// Custom raw text if source_type == Custom
    pub raw_text: String,
}

impl Default for WorksCitedEntry {
    fn default() -> Self {
        Self {
            id: generate_id(),
            source_type: SourceType::BookOrStandalone,
            author: String::new(),
            title_of_source: String::new(),
            container_title: String::new(),
            other_contributors: String::new(),
            version: String::new(),
            number: String::new(),
            publisher: String::new(),
            pub_date: String::new(),
            location: String::new(),
            raw_text: String::new(),
        }
    }
}

impl WorksCitedEntry {
    pub fn new_empty() -> Self {
        Self::default()
    }

    /// Sort key according to MLA 9 rules (Author last name, or title if no author)
    pub fn sort_key(&self) -> String {
        let key = if !self.author.trim().is_empty() {
            self.author.trim().to_lowercase()
        } else if !self.title_of_source.trim().is_empty() {
            // MLA ignores leading "A ", "An ", "The " for alphabetizing
            let t = self.title_of_source.trim().to_lowercase();
            if let Some(stripped) = t.strip_prefix("the ") {
                stripped.to_string()
            } else if let Some(stripped) = t.strip_prefix("a ") {
                stripped.to_string()
            } else if let Some(stripped) = t.strip_prefix("an ") {
                stripped.to_string()
            } else {
                t
            }
        } else if !self.raw_text.trim().is_empty() {
            self.raw_text.trim().to_lowercase()
        } else {
            "zzz".to_string()
        };
        key
    }

    /// Generates short in-text citation suggestion, e.g. "(Morrison 42)" or "(\"Art of Fiction\" 12)"
    pub fn in_text_citation_prompt(&self, page: &str) -> String {
        let page_part = if page.trim().is_empty() {
            String::new()
        } else {
            format!(" {}", page.trim())
        };

        if !self.author.trim().is_empty() {
            // Extract author last name (before comma if "Last, First")
            let last_name = self.author.split(',').next().unwrap_or(&self.author).trim();
            format!("({}{})", last_name, page_part)
        } else if !self.title_of_source.trim().is_empty() {
            let short_title = self.title_of_source.trim();
            let truncated = if short_title.len() > 25 {
                let end = short_title
                    .char_indices()
                    .map(|(i, _)| i)
                    .nth(25)
                    .unwrap_or(short_title.len());
                format!("{}...", &short_title[..end])
            } else {
                short_title.to_string()
            };

            match self.source_type {
                SourceType::BookOrStandalone => format!("(*{}*{})", truncated, page_part),
                _ => format!("(\"{}\"{})", truncated, page_part),
            }
        } else {
            format!("(Author{})", page_part)
        }
    }

    /// Renders text with Markdown markers for italics: *Title*
    pub fn format_markdown(&self) -> String {
        if self.source_type == SourceType::Custom && !self.raw_text.trim().is_empty() {
            return self.raw_text.trim().to_string();
        }

        let mut parts = Vec::new();

        // 1. Author.
        if !self.author.trim().is_empty() {
            let a = ensure_trailing_period(self.author.trim());
            parts.push(a);
        }

        // 2. Title of source.
        if !self.title_of_source.trim().is_empty() {
            let s_title = self.title_of_source.trim();
            match self.source_type {
                SourceType::BookOrStandalone => {
                    // Italicized, with trailing period inside or outside
                    parts.push(format!("*{}.*", s_title.trim_end_matches('.')));
                }
                _ => {
                    // Quotation marks with period inside
                    let inside = ensure_trailing_period(s_title);
                    parts.push(format!("\"{}\"", inside));
                }
            }
        }

        // 3. Container & following elements (joined by commas, terminating with period)
        let mut container_parts = Vec::new();

        if !self.container_title.trim().is_empty() {
            container_parts.push(format!(
                "*{}*",
                self.container_title.trim().trim_end_matches([',', '.'])
            ));
        }
        if !self.other_contributors.trim().is_empty() {
            container_parts.push(
                self.other_contributors
                    .trim()
                    .trim_end_matches([',', '.'])
                    .to_string(),
            );
        }
        if !self.version.trim().is_empty() {
            container_parts.push(self.version.trim().trim_end_matches([',', '.']).to_string());
        }
        if !self.number.trim().is_empty() {
            container_parts.push(self.number.trim().trim_end_matches([',', '.']).to_string());
        }
        if !self.publisher.trim().is_empty() {
            container_parts.push(
                self.publisher
                    .trim()
                    .trim_end_matches([',', '.'])
                    .to_string(),
            );
        }
        if !self.pub_date.trim().is_empty() {
            container_parts.push(
                self.pub_date
                    .trim()
                    .trim_end_matches([',', '.'])
                    .to_string(),
            );
        }
        if !self.location.trim().is_empty() {
            container_parts.push(
                self.location
                    .trim()
                    .trim_end_matches([',', '.'])
                    .to_string(),
            );
        }

        if !container_parts.is_empty() {
            let joined = container_parts.join(", ") + ".";
            parts.push(joined);
        }

        parts.join(" ")
    }
}

fn ensure_trailing_period(s: &str) -> String {
    let trimmed = s.trim();
    if trimmed.ends_with('.') || trimmed.ends_with('?') || trimmed.ends_with('!') {
        trimmed.to_string()
    } else {
        format!("{}.", trimmed)
    }
}

fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{:x}", nanos)
}
