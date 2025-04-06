use super::common::{LintContext, LintError, Rule};
use pulldown_cmark::Event;

pub struct NoConsecutiveEmptyLinesRule {}

impl NoConsecutiveEmptyLinesRule {
    pub fn new() -> Self {
        Self {}
    }
}

impl Rule for NoConsecutiveEmptyLinesRule {
    fn id(&self) -> &'static str { "MD012" }
    fn name(&self) -> &'static str { "No Consecutive Empty Lines" }
    fn description(&self) -> &'static str { "Ensures no more than one consecutive empty line." }

    // NOTE: This check function assumes the linter calls it once per line,
    // providing line-specific info in the LintContext.
    // It ignores the `event` parameter in this specific rule.

    fn check(&self, event: &Event<'_>, context: &LintContext) -> Option<LintError> {
        // Check if the current line is blank AND the previous line was also blank
        if context.current_line_is_blank && context.previous_line_was_blank {
            Some(LintError {
                line: context.current_line_number,
                column: 0, // Column is not specific for this rule
                message: "Multiple consecutive blank lines found (MD012)".to_string(),
                rule_id: self.id().to_string(),
            })
        } else {
        None
        }
    }
}