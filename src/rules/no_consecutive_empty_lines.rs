use super::common::{LineRule, LintError};
use std::path::Path;

pub struct NoConsecutiveEmptyLinesRule;

impl LineRule for NoConsecutiveEmptyLinesRule {
    fn id(&self) -> &'static str {
        "MD012"
    }

    fn check_line(
        &self,
        file_path: &Path,
        line: &str,
        line_number: usize,
        previous_line_was_blank: bool,
    ) -> Option<LintError> {
        // Current line is blank AND the previous line was also blank
        if line.trim().is_empty() && previous_line_was_blank {
            Some(LintError {
                file_path: file_path.to_path_buf(),
                line: line_number,
                message: "Multiple consecutive blank lines found (MD012)".to_string(),
                rule_id: self.id().to_string(),
            })
        } else {
            None
        }
    }
}
