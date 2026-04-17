use pulldown_cmark::{Event, Parser, Tag};
use std::path::Path;

// setup module system rules
use crate::rules::common::{LintContext, LintError};
use crate::rules::get_rules;

/// Lints the provided Markdown content against defined rules.
///
/// Currently, it checks for empty links (e.g., `[]()`).
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use md_check::linter::lint;
///
/// let content = "This text contains an [empty link]().";
/// let file_path = Path::new("test.md");
///
/// let errors = lint(content, file_path);
///
/// assert_eq!(errors.len(), 1);
/// assert_eq!(errors[0].rule_id, "NO_EMPTY_LINKS");
/// assert_eq!(errors[0].message, "Empty link URL found");
/// ```
pub fn lint(content: &str, file_path: &Path) -> Vec<LintError> {
    let parser = Parser::new(content);
    let mut errors = Vec::new();
    let rules = get_rules();

    let lines: Vec<&str> = content.lines().collect();

    // ==========================================
    // 1: check lines (MD012)
    // ==========================================
    let mut previous_line_was_blank = false;
    for (idx, line) in lines.iter().enumerate() {
        let current_line_number = idx + 1;
        let current_line_is_blank = line.trim().is_empty();

        let context = LintContext {
            file_path: file_path.to_path_buf(),
            current_line_number,
            current_line_is_blank,
            previous_line_was_blank,
            line_text: line.to_string(),
        };

        let dummy_event = Event::Text("".into());

        for rule in &rules {
            let rule_id = rule.id();
            // run only rules that check lines (MD012)
            if (rule_id == "MD012" || rule_id == "LINE_TOO_LONG")
                && let Some(error) = rule.check(&dummy_event, &context)
            {
                errors.push(error);
            }
        }
        previous_line_was_blank = current_line_is_blank;
    }

    // ==========================================
    // 2: check AST (NO_EMPTY_LINKS etc.)
    // ==========================================
    for (line_num, event) in (1..).zip(parser) {
        let mut event_line = 1;
        if let Event::Start(Tag::Link { dest_url, .. }) = &event {
            for (idx, line) in lines.iter().enumerate() {
                if line.contains(dest_url.as_ref()) {
                    event_line = idx + 1;
                    break;
                }
            }
        } else {
            event_line = line_num;
        }

        let context = LintContext {
            file_path: file_path.to_path_buf(),
            current_line_number: event_line,
            current_line_is_blank: false,
            previous_line_was_blank: false,
            line_text: String::new(),
        };

        for rule in &rules {
            let rule_id = rule.id();
            // run only AST rules
            if rule_id != "MD012"
                && rule_id != "LINE_TOO_LONG"
                && let Some(mut error) = rule.check(&event, &context)
            {
                error.line = event_line;
                errors.push(error);
            }
        }
    }

    // deduplicate and sort errors by line number
    use std::collections::HashSet;
    let mut seen = HashSet::new();
    errors.retain(|e| {
        let key = (
            e.file_path.clone(),
            e.line,
            e.rule_id.clone(),
            e.message.clone(),
        );
        seen.insert(key)
    });

    errors.sort_by_key(|e| e.line);
    errors
}