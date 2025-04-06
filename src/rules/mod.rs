pub mod common;
mod no_empty_links;
mod no_html;
mod no_consecutive_empty_lines;

pub use no_empty_links::NoEmptyLinksRule;
pub use no_html::NoHtmlRule;
pub use no_consecutive_empty_lines::NoConsecutiveEmptyLinesRule; 

// Function to get all available rules
pub fn get_rules() -> Vec<Box<dyn common::Rule>> {
    vec![
        Box::new(NoEmptyLinksRule::new()),
        Box::new(NoHtmlRule::new()),
        Box::new(NoConsecutiveEmptyLinesRule::new()),
    ]
}
