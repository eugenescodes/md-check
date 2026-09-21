use pulldown_cmark::Event;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct LintError {
    pub file_path: PathBuf,
    pub line: usize,
    pub message: String,
    pub rule_id: String,
}

/// A rule that inspects the file line by line (blank lines, line length, ...).
pub trait LineRule {
    fn id(&self) -> &'static str;

    /// Checks a single line. `previous_line_was_blank` lets stateful rules
    /// (e.g. MD012) work without keeping internal state.
    fn check_line(
        &self,
        file_path: &Path,
        line: &str,
        line_number: usize,
        previous_line_was_blank: bool,
    ) -> Option<LintError>;
}

/// A rule that inspects AST events produced by pulldown-cmark.
pub trait EventRule {
    fn id(&self) -> &'static str;

    /// `line_number` is the exact 1-based line of the event, derived from the
    /// parser's byte offsets.
    fn check_event(
        &self,
        file_path: &Path,
        event: &Event<'_>,
        line_number: usize,
    ) -> Option<LintError>;
}
