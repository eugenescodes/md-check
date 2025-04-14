use assert_cmd::Command;
use predicates::prelude::*;
use std::error::Error;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_cli_no_files() -> Result<(), Box<dyn Error>> {
    let temp_dir = TempDir::new()?;
    let empty_dir = temp_dir.path();

    let mut cmd = Command::cargo_bin("md-check")?;
    cmd.arg(empty_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("No Markdown files found"));

    Ok(())
}

#[test]
fn test_multiple_files() -> Result<(), Box<dyn Error>> {
    let temp_dir = TempDir::new()?;

    // Create two markdown files
    let file1_path = temp_dir.path().join("test1.md");
    let file2_path = temp_dir.path().join("test2.md");
    fs::write(&file1_path, "# Valid markdown 1")?;
    fs::write(&file2_path, "# Valid markdown 2")?;

    let mut cmd = Command::cargo_bin("md-check")?;
    cmd.arg(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Found 2 Markdown files"));

    Ok(())
}

#[test]
fn test_skip_links_flag() -> Result<(), Box<dyn Error>> {
    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path().join("test.md");
    fs::write(&file_path, "[Test](https://example.com)")?;

    let mut cmd = Command::cargo_bin("md-check")?;
    cmd.arg(&file_path)
        .arg("--skip-links")
        .assert()
        .success()
        // Should not see any link checking output
        .stdout(
            predicate::str::contains("Found 1 Markdown files")
                .and(predicate::str::contains("Analyzing")),
        );

    Ok(())
}

#[test]
fn test_invalid_file_path() -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::cargo_bin("md-check")?;
    cmd.arg("nonexistent.md")
        .assert()
        .success()
        .stdout(predicate::str::contains("No Markdown files found"));

    Ok(())
}
