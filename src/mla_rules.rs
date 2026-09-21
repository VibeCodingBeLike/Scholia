use crate::model::{MlaBlock, MlaDocument};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleSeverity {
    Error,
    Warning,
    Suggestion,
}

impl RuleSeverity {
    pub fn badge_label(&self) -> &'static str {
        match self {
            RuleSeverity::Error => "VIOLATION",
            RuleSeverity::Warning => "WARNING",
            RuleSeverity::Suggestion => "RECOMMENDED",
        }
    }
}

#[derive(Debug, Clone)]
pub struct RuleIssue {
    pub rule_id: &'static str,
    pub title: &'static str,
    pub description: String,
    pub severity: RuleSeverity,
    pub can_auto_fix: bool,
}

#[derive(Debug, Clone, Default)]
pub struct ComplianceReport {
    pub score_percentage: u32,
    pub issues: Vec<RuleIssue>,
    pub passed_rules_count: usize,
    pub total_rules_checked: usize,
}

impl ComplianceReport {
    pub fn is_fully_compliant(&self) -> bool {
        self.issues
            .iter()
            .all(|i| i.severity == RuleSeverity::Suggestion)
    }
}

pub struct MlaLinter;

impl MlaLinter {
    pub fn inspect(doc: &MlaDocument) -> ComplianceReport {
        let mut issues = Vec::new();
        let mut passed = 0;
        let mut total = 0;

        // --- Rule 1: Student Name ---
        total += 1;
        if doc.header.student_name.trim().is_empty() {
            issues.push(RuleIssue {
                rule_id: "HDR_STUDENT_NAME",
                title: "Missing Student Name",
                description:
                    "MLA format requires the student's full name at the top left of the first page."
                        .to_string(),
                severity: RuleSeverity::Error,
                can_auto_fix: false,
            });
        } else {
            passed += 1;
        }

        // --- Rule 2: Instructor Name ---
        total += 1;
        if doc.header.instructor_name.trim().is_empty() {
            issues.push(RuleIssue {
                rule_id: "HDR_INSTRUCTOR_NAME",
                title: "Missing Instructor Name",
                description:
                    "MLA requires the instructor's name (e.g., 'Professor Smith' or 'Dr. Johnson')."
                        .to_string(),
                severity: RuleSeverity::Error,
                can_auto_fix: false,
            });
        } else {
            passed += 1;
        }

        // --- Rule 3: Course Title/Number ---
        total += 1;
        if doc.header.course.trim().is_empty() {
            issues.push(RuleIssue {
                rule_id: "HDR_COURSE",
                title: "Missing Course Name",
                description:
                    "MLA requires the course designation (e.g., 'ENG 101' or 'English 102')."
                        .to_string(),
                severity: RuleSeverity::Error,
                can_auto_fix: false,
            });
        } else {
            passed += 1;
        }

        // --- Rule 4: Date Format (Day Month Year) ---
        total += 1;
        let date_str = doc.header.date.trim();
        if date_str.is_empty() {
            issues.push(RuleIssue {
                rule_id: "HDR_DATE_MISSING",
                title: "Missing Date",
                description: "MLA requires the date of submission in Day Month Year order."
                    .to_string(),
                severity: RuleSeverity::Error,
                can_auto_fix: false,
            });
        } else if !is_valid_mla_date(date_str) {
            issues.push(RuleIssue {
                rule_id: "HDR_DATE_FORMAT",
                title: "Non-standard Date Format",
                description: format!(
                    "Date '{}' does not follow MLA 9 style: 'Day Month Year' (e.g. '20 September 2026'). Avoid slashes or Month/Day order.",
                    date_str
                ),
                severity: RuleSeverity::Warning,
                can_auto_fix: false,
            });
        } else {
            passed += 1;
        }

        // --- Rule 5: Running Header ---
        total += 1;
        if doc.header.derived_last_name().is_empty() {
            issues.push(RuleIssue {
                rule_id: "HDR_RUNNING_HEAD",
                title: "Missing Running Header Last Name",
                description: "MLA 9 requires author's last name followed by page number at top right of every page.".to_string(),
                severity: RuleSeverity::Warning,
                can_auto_fix: false,
            });
        } else {
            passed += 1;
        }

        // --- Rule 6: Paper Title ---
        total += 1;
        let title_trim = doc.title.trim();
        if title_trim.is_empty() {
            issues.push(RuleIssue {
                rule_id: "TITLE_MISSING",
                title: "Missing Paper Title",
                description: "Every MLA research paper must have a centered title.".to_string(),
                severity: RuleSeverity::Error,
                can_auto_fix: false,
            });
        } else {
            if title_trim.ends_with('.') {
                issues.push(RuleIssue {
                    rule_id: "TITLE_PERIOD",
                    title: "Period at End of Title",
                    description: "MLA paper titles should not end with a period.".to_string(),
                    severity: RuleSeverity::Warning,
                    can_auto_fix: false,
                });
            }
            let expected_title_case = crate::model::to_mla_title_case(title_trim);
            if title_trim != expected_title_case {
                issues.push(RuleIssue {
                    rule_id: "TITLE_CASE",
                    title: "Title Capitalization",
                    description: format!(
                        "MLA requires Title Case (e.g. '{}'). Short words like 'is', 'the', and prepositions are not capitalized.",
                        expected_title_case
                    ),
                    severity: RuleSeverity::Warning,
                    can_auto_fix: true,
                });
            }
            passed += 1;
        }

        // --- Rule 7: Block Quotes Length ---
        total += 1;
        let mut short_blockquotes = 0;
        for block in &doc.blocks {
            if let MlaBlock::BlockQuote { text, .. } = block {
                let wc = text.split_whitespace().count();
                if wc > 0 && wc < 30 {
                    short_blockquotes += 1;
                }
            }
        }
        if short_blockquotes > 0 {
            issues.push(RuleIssue {
                rule_id: "BLOCKQUOTE_SHORT",
                title: "Short Block Quote Detected",
                description: format!(
                    "Found {} block quote(s) under 30 words. MLA 9 specifies that quotations under 4 lines of prose should remain inline with quotation marks.",
                    short_blockquotes
                ),
                severity: RuleSeverity::Suggestion,
                can_auto_fix: false,
            });
        } else {
            passed += 1;
        }

        // --- Rule 8: In-text Citation Punctuation ---
        total += 1;
        let mut citation_punct_issues = 0;
        for block in &doc.blocks {
            if let MlaBlock::Paragraph { text, .. } = block {
                // Look for common error: period before parenthetical citation: e.g. "something." (Smith 12)
                if text.contains(".\" (") || text.contains(". (") {
                    citation_punct_issues += 1;
                }
            }
        }
        if citation_punct_issues > 0 {
            issues.push(RuleIssue {
                rule_id: "CITATION_PUNCTUATION",
                title: "Period Placed Before Parenthetical Citation",
                description: "In MLA format, for standard inline quotes, the period belongs after the parenthetical citation: e.g., \"quotation\" (Smith 42). Not \"quotation.\" (Smith 42).".to_string(),
                severity: RuleSeverity::Warning,
                can_auto_fix: false,
            });
        } else {
            passed += 1;
        }

        // --- Rule 9: Works Cited Alphabetization ---
        total += 1;
        if doc.works_cited.len() > 1 {
            let mut is_sorted = true;
            for i in 0..doc.works_cited.len() - 1 {
                if doc.works_cited[i].sort_key() > doc.works_cited[i + 1].sort_key() {
                    is_sorted = false;
                    break;
                }
            }
            if !is_sorted {
                issues.push(RuleIssue {
                    rule_id: "WORKS_CITED_UNSORTED",
                    title: "Works Cited Not Alphabetized",
                    description: "MLA 9 requires Works Cited entries to be alphabetized by author's last name or primary title.".to_string(),
                    severity: RuleSeverity::Warning,
                    can_auto_fix: false,
                });
            } else {
                passed += 1;
            }
        } else {
            passed += 1;
        }

        // Calculate score
        let error_count = issues
            .iter()
            .filter(|i| i.severity == RuleSeverity::Error)
            .count();
        let warning_count = issues
            .iter()
            .filter(|i| i.severity == RuleSeverity::Warning)
            .count();

        let penalty = (error_count * 20) + (warning_count * 10);
        let score = if penalty >= 100 {
            0
        } else {
            100 - penalty as u32
        };

        ComplianceReport {
            score_percentage: score,
            issues,
            passed_rules_count: passed,
            total_rules_checked: total,
        }
    }

    pub fn auto_fix(doc: &mut MlaDocument, rule_id: &str) {
        match rule_id {
            "TITLE_CASE" | "TITLE_ALL_CAPS" => {
                doc.title = crate::model::to_mla_title_case(&doc.title);
                doc.is_dirty = true;
            }
            _ => {}
        }
    }
}

fn is_valid_mla_date(s: &str) -> bool {
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() != 3 {
        return false;
    }
    // Part 1: Day (number 1-31)
    if let Ok(day) = parts[0].parse::<u32>() {
        if !(1..=31).contains(&day) {
            return false;
        }
    } else {
        return false;
    }

    // Part 2: Month name
    let months = [
        "january",
        "february",
        "march",
        "april",
        "may",
        "june",
        "july",
        "august",
        "september",
        "october",
        "november",
        "december",
        "jan.",
        "feb.",
        "mar.",
        "apr.",
        "may",
        "june",
        "july",
        "aug.",
        "sept.",
        "oct.",
        "nov.",
        "dec.",
    ];
    let month_lower = parts[1].to_lowercase();
    if !months.contains(&month_lower.as_str()) {
        return false;
    }

    // Part 3: Year (4 digit number)
    if let Ok(year) = parts[2].parse::<u32>() {
        if !(1900..=2100).contains(&year) {
            return false;
        }
    } else {
        return false;
    }

    true
}
