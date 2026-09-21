use super::common::{LineRule, LintError};
use std::path::Path;

pub struct LineTooLongRule;

impl LineRule for LineTooLongRule {
    fn id(&self) -> &'static str {
        "LINE_TOO_LONG"
    }

    fn check_line(
        &self,
        file_path: &Path,
        line: &str,
        line_number: usize,
        _previous_line_was_blank: bool,
    ) -> Option<LintError> {
        // Check the length of the line in characters, not bytes
        if line.chars().count() > 100 {
            Some(LintError {
                file_path: file_path.to_path_buf(),
                line: line_number,
                message: "Line exceeds 100 characters limit".to_string(),
                rule_id: self.id().to_string(),
            })
        } else {
            None
        }
    }
}
