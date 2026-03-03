use super::common::{LintContext, LintError, Rule};
use pulldown_cmark::Event;

pub struct LineTooLongRule;

impl Default for LineTooLongRule {
    fn default() -> Self {
        Self::new()
    }
}

impl LineTooLongRule {
    pub fn new() -> Self {
        Self
    }
}

impl Rule for LineTooLongRule {
    fn id(&self) -> &'static str {
        "LINE_TOO_LONG"
    }
    fn name(&self) -> &'static str {
        "Line Too Long"
    }
    fn description(&self) -> &'static str {
        "Ensures lines do not exceed 100 characters."
    }

    fn check(&self, _event: &Event<'_>, context: &LintContext) -> Option<LintError> {
        // check length of the line text in characters, not bytes
        if context.line_text.chars().count() > 100 {
            Some(LintError {
                file_path: context.file_path.clone(),
                line: context.current_line_number,
                message: "Line exceeds 100 characters limit".to_string(),
                rule_id: self.id().to_string(),
            })
        } else {
            None
        }
    }
}
