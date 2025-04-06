use super::common::{LintContext, LintError, Rule};
use pulldown_cmark::{Event, Tag};

pub struct NoEmptyLinksRule;

impl NoEmptyLinksRule {
    pub fn new() -> Self {
        Self
    }
}

impl Rule for NoEmptyLinksRule {
    fn id(&self) -> &'static str {
        "NO_EMPTY_LINKS"
    }

    fn name(&self) -> &'static str {
        "No Empty Links"
    }

    fn description(&self) -> &'static str {
        "Ensures that all links have a non-empty URL"
    }

    fn check(&self, event: &Event<'_>, context: &LintContext) -> Option<LintError> {
        if let Event::Start(Tag::Link { dest_url, .. }) = event {
            if dest_url.is_empty() {
                return Some(LintError {
                    file_path: context.file_path.to_path_buf(),
                    line: 0,
                    message: "Empty link URL found".to_string(),
                    rule_id: self.id().to_string(),
                });
            }
        }
        None
    }
}
