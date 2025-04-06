use pulldown_cmark::{Event, Parser, Tag};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct LintError {
    pub file_path: PathBuf,
    pub line: usize,
    pub message: String,
    pub rule_id: String,
}

pub trait Rule {
    fn check(&self, event: &Event<'_>, file_path: &Path) -> Option<LintError>;
}

struct NoEmptyLinksRule;

impl Rule for NoEmptyLinksRule {
    fn check(&self, event: &Event<'_>, file_path: &Path) -> Option<LintError> {
        if let Event::Start(Tag::Link { dest_url, .. }) = event {
            if dest_url.is_empty() {
                return Some(LintError {
                    file_path: file_path.to_path_buf(),
                    line: 0,
                    message: "Empty link URL found".to_string(),
                    rule_id: "NO_EMPTY_LINKS".to_string(),
                });
            }
        }
        None
    }
}

pub fn lint(content: &str, file_path: &Path) -> Vec<LintError> {
    let parser = Parser::new(content);
    let mut errors = Vec::new();
    let rules: Vec<Box<dyn Rule>> = vec![Box::new(NoEmptyLinksRule)];

    for event in parser {
        for rule in &rules {
            if let Some(error) = rule.check(&event, file_path) {
                errors.push(error);
            }
        }
    }

    errors
}
