use anyhow::Result;
use clap::Parser;
use colored::*;
use std::fs;
use std::path::PathBuf;

use md_check::link_checker;
use md_check::linter;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Files or directories to check
    #[arg(required = true)]
    paths: Vec<PathBuf>,

    /// Skip link checking
    #[arg(long)]
    skip_links: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let mut markdown_files = Vec::new();

    // Find all Markdown files
    for path in args.paths {
        if path.is_dir() {
            for entry in walkdir::WalkDir::new(path)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|e| !e.file_type().is_dir())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
            {
                markdown_files.push(entry.path().to_path_buf());
            }
        } else if path.is_file() && path.extension().is_some_and(|ext| ext == "md") {
            markdown_files.push(path);
        }
    }

    if markdown_files.is_empty() {
        println!("{}", "No Markdown files found.".yellow());
        return Ok(());
    }

    println!(
        "{} {} Markdown files.",
        "Found".green(),
        markdown_files.len()
    );

    let mut all_links = Vec::new();
    let mut lint_errors = Vec::new();

    // Analyze files: a single parsing pass per file collects both lint errors
    // and links to check
    for file_path in &markdown_files {
        println!("{} {}", "Analyzing".cyan(), file_path.display());
        match fs::read_to_string(file_path) {
            Ok(content) => {
                let analysis = linter::analyze(&content, file_path);
                lint_errors.extend(analysis.lint_errors);

                if !args.skip_links {
                    all_links.extend(analysis.links);
                }
            }
            Err(e) => {
                eprintln!("{}: {} - {}", "Error".red(), file_path.display(), e);
            }
        }
    }

    let mut has_problems = false;

    // Check links if not skipped
    if !args.skip_links && !all_links.is_empty() {
        let results = link_checker::check_links(all_links).await;
        let formatted_errors = link_checker::format_check_results(&results);

        if !formatted_errors.is_empty() {
            println!("\n{}", "Problematic links:".red());
            for error in formatted_errors {
                println!("{}", error);
            }
            has_problems = true;
        }
    }

    // Print lint errors
    if !lint_errors.is_empty() {
        println!("\n{}", "Style errors:".red());
        for error in &lint_errors {
            println!(
                "[{}] {}:{} {}",
                error.rule_id.yellow(),
                error.file_path.display(),
                error.line,
                error.message
            );
        }
        has_problems = true;
    }

    // Non-zero exit code for CI when any problem was found
    if has_problems {
        std::process::exit(1);
    }

    Ok(())
}
