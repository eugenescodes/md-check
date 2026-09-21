pub mod common;
mod line_too_long;
mod no_consecutive_empty_lines;
mod no_empty_links;
mod no_html;

use common::{EventRule, LineRule};

pub use line_too_long::LineTooLongRule;
pub use no_consecutive_empty_lines::NoConsecutiveEmptyLinesRule;
pub use no_empty_links::NoEmptyLinksRule;
pub use no_html::NoHtmlRule;

/// Rules that inspect the file line by line.
pub fn get_line_rules() -> Vec<Box<dyn LineRule>> {
    vec![
        Box::new(NoConsecutiveEmptyLinesRule),
        Box::new(LineTooLongRule),
    ]
}

/// Rules that inspect pulldown-cmark AST events.
pub fn get_event_rules() -> Vec<Box<dyn EventRule>> {
    vec![Box::new(NoEmptyLinksRule), Box::new(NoHtmlRule)]
}
