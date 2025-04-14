use super::common::{LintContext, LintError, Rule};
use pulldown_cmark::Event;

pub struct NoHtmlRule;

impl NoHtmlRule {
    pub fn new() -> Self {
        Self
    }
}

impl Rule for NoHtmlRule {
    fn id(&self) -> &'static str {
        "NO_HTML"
    }

    fn name(&self) -> &'static str {
        "No HTML"
    }

    fn description(&self) -> &'static str {
        "Ensures that no raw HTML is used in markdown"
    }

    fn check(&self, event: &Event<'_>, context: &LintContext) -> Option<LintError> {
        match event {
            Event::Html(_) => Some(LintError {
                file_path: context.file_path.to_path_buf(),
                line: 0,
                message: "Raw HTML found in markdown".to_string(),
                rule_id: self.id().to_string(),
            }),
            _ => None,
        }
    }
}
