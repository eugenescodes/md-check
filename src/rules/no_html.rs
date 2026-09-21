use super::common::{EventRule, LintError};
use pulldown_cmark::Event;
use std::path::Path;

pub struct NoHtmlRule;

impl EventRule for NoHtmlRule {
    fn id(&self) -> &'static str {
        "NO_HTML"
    }

    fn check_event(
        &self,
        file_path: &Path,
        event: &Event<'_>,
        line_number: usize,
    ) -> Option<LintError> {
        if matches!(event, Event::Html(_)) {
            Some(LintError {
                file_path: file_path.to_path_buf(),
                line: line_number,
                message: "Raw HTML found in markdown".to_string(),
                rule_id: self.id().to_string(),
            })
        } else {
            None
        }
    }
}
