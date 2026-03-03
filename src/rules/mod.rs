pub mod common;
mod line_too_long;
mod no_consecutive_empty_lines;
mod no_empty_links;
mod no_html;

pub use line_too_long::LineTooLongRule;
pub use no_consecutive_empty_lines::NoConsecutiveEmptyLinesRule;
pub use no_empty_links::NoEmptyLinksRule;
pub use no_html::NoHtmlRule;

// Function to get all available rules
pub fn get_rules() -> Vec<Box<dyn common::Rule>> {
    vec![
        Box::new(NoEmptyLinksRule::new()),
        Box::new(NoHtmlRule::new()),
        Box::new(NoConsecutiveEmptyLinesRule::new()),
        Box::new(LineTooLongRule::new()),
    ]
}
