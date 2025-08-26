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
        if let Event::Start(Tag::Link { dest_url, .. }) = event
            && dest_url.is_empty()
        {
            return Some(LintError {
                file_path: file_path.to_path_buf(),
                line: 0,
                message: "Empty link URL found".to_string(),
                rule_id: "NO_EMPTY_LINKS".to_string(),
            });
        }
        None
    }
}

pub fn lint(content: &str, file_path: &Path) -> Vec<LintError> {
    let parser = Parser::new(content);
    let mut errors = Vec::new();
    let rules: Vec<Box<dyn Rule>> = vec![Box::new(NoEmptyLinksRule)];

    // Split content into lines for line number tracking
    let lines: Vec<&str> = content.lines().collect();
    let mut current_line = 1;

    for event in parser {
        // Try to match event text to a line for better line number reporting
        let mut event_line = 1;
        if let Event::Start(Tag::Link { dest_url, .. }) = &event {
            for (idx, line) in lines.iter().enumerate() {
                if line.contains(dest_url.as_ref()) {
                    event_line = idx + 1;
                    break;
                }
            }
        } else {
            event_line = current_line;
        }

        for rule in &rules {
            if let Some(mut error) = rule.check(&event, file_path) {
                error.line = event_line;
                errors.push(error);
            }
        }
        current_line += 1;
    }

    // Deduplicate errors by (file_path, line, rule_id, message)
    use std::collections::HashSet;
    let mut seen = HashSet::new();
    errors.retain(|e| {
        let key = (
            e.file_path.clone(),
            e.line,
            e.rule_id.clone(),
            e.message.clone(),
        );
        seen.insert(key)
    });

    errors
}
