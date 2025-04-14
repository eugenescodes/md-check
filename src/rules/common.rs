use pulldown_cmark::Event;
use std::path::PathBuf;

pub struct LintError {
    pub file_path: PathBuf,
    pub line: usize,
    pub message: String,
    pub rule_id: String,
}

pub struct LintContext {
    pub file_path: PathBuf,
}

pub trait Rule {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn check(&self, event: &Event<'_>, context: &LintContext) -> Option<LintError>;
}
