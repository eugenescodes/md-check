use super::common::{EventRule, LintError};
use pulldown_cmark::{Event, Tag};
use std::path::Path;

pub struct NoEmptyLinksRule;

impl EventRule for NoEmptyLinksRule {
    fn id(&self) -> &'static str {
        "NO_EMPTY_LINKS"
    }

    fn check_event(
        &self,
        file_path: &Path,
        event: &Event<'_>,
        line_number: usize,
    ) -> Option<LintError> {
        if let Event::Start(Tag::Link { dest_url, .. }) = event
            && dest_url.is_empty()
        {
            return Some(LintError {
                file_path: file_path.to_path_buf(),
                line: line_number,
                message: "Empty link URL found".to_string(),
                rule_id: self.id().to_string(),
            });
        }
        None
    }
}
