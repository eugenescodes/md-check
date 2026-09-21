use pulldown_cmark::{Event, Parser, Tag};
use std::path::Path;
use url::Url;

use crate::link_checker::LinkInfo;
use crate::rules::common::LintError;
use crate::rules::{get_event_rules, get_line_rules};

/// Result of analyzing one Markdown file: lint errors plus links to check.
#[derive(Debug, Default)]
pub struct Analysis {
    pub lint_errors: Vec<LintError>,
    pub links: Vec<LinkInfo>,
}

/// Byte offset of the start of every line; line `k` (1-based) starts at
/// `starts[k - 1]`.
fn line_starts(content: &str) -> Vec<usize> {
    let mut starts = vec![0];
    for (offset, byte) in content.bytes().enumerate() {
        if byte == b'\n' {
            starts.push(offset + 1);
        }
    }
    starts
}

/// 1-based line number containing the given byte offset.
fn line_number(starts: &[usize], offset: usize) -> usize {
    starts.partition_point(|&start| start <= offset)
}

/// Analyzes Markdown content in a single parsing pass.
///
/// Runs the line-based rules (MD012, LINE_TOO_LONG), the AST-based rules
/// (NO_EMPTY_LINKS, NO_HTML) and collects all http(s) links for checking.
/// Event line numbers are derived from the parser's byte offsets, so they are
/// exact even when several events occur on the same line.
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use md_check::linter::analyze;
///
/// let content = "This text contains an [empty link]().";
/// let file_path = Path::new("test.md");
///
/// let analysis = analyze(content, file_path);
///
/// assert_eq!(analysis.lint_errors.len(), 1);
/// assert_eq!(analysis.lint_errors[0].rule_id, "NO_EMPTY_LINKS");
/// assert_eq!(analysis.lint_errors[0].message, "Empty link URL found");
/// assert_eq!(analysis.lint_errors[0].line, 1);
/// ```
pub fn analyze(content: &str, file_path: &Path) -> Analysis {
    let starts = line_starts(content);
    let mut analysis = Analysis::default();

    let line_rules = get_line_rules();
    let event_rules = get_event_rules();

    // 1: line-based rules (MD012, LINE_TOO_LONG)
    let mut previous_line_was_blank = false;
    for (idx, line) in content.lines().enumerate() {
        let line_number = idx + 1;

        for rule in &line_rules {
            if let Some(error) =
                rule.check_line(file_path, line, line_number, previous_line_was_blank)
            {
                analysis.lint_errors.push(error);
            }
        }

        previous_line_was_blank = line.trim().is_empty();
    }

    // 2: AST rules and link extraction in a single pass
    for (event, range) in Parser::new(content).into_offset_iter() {
        let line = line_number(&starts, range.start);

        if let Event::Start(Tag::Link { dest_url, .. }) = &event {
            let url_str = dest_url.to_string();
            if (url_str.starts_with("http://") || url_str.starts_with("https://"))
                && Url::parse(&url_str).is_ok()
            {
                analysis.links.push(LinkInfo {
                    url: url_str,
                    file_path: file_path.to_path_buf(),
                });
            }
        }

        for rule in &event_rules {
            if let Some(error) = rule.check_event(file_path, &event, line) {
                analysis.lint_errors.push(error);
            }
        }
    }

    // Deduplicate identical errors (same line, rule and message) and sort by line
    analysis
        .lint_errors
        .sort_by(|a, b| (a.line, &a.rule_id, &a.message).cmp(&(b.line, &b.rule_id, &b.message)));
    // Use owned keys so no borrowed fields escape the deduplication closure.
    analysis
        .lint_errors
        .dedup_by_key(|error| (error.line, error.rule_id.clone(), error.message.clone()));

    analysis
}
