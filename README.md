# Markdown Link Checker

A command-line tool that validates markdown files by checking for broken links and applying linting rules.

## Features

- Recursively finds all markdown files in specified directories
- Checks for broken links in markdown files
- Validates links
- Shows real-time progress with colored output
- Provides a summary of link check results
- Basic markdown linting capabilities and capability add new rules
- Supports multiple files and directories
- Supports GitHub action and pre-commit

## Installation

### Prerequisites

Install on local machine:

- Rust 1.85 +
- Cargo (Rust's package manager)

### Building from source

```bash
git clone https://github.com/yourusername/md-check
cd md-check
cargo build --release
```

The compiled binary will be available at target/release/md-check

## Examples for use

```bash
# Check a single file

md-check README.md

# Check multiple files

md-check file1.md file2.md

# Check all markdown files in a directory

md-check .
or
md-check ./folder_name/

# Check multiple directories

md-check ./folder_name_1/ ./folder_name_2/

# Skip link checking

md-check --skip-links README.md
```

Output

The tool provides colored output showing:

- Total number of files and links found
- Real-time progress of link checking
- HTTP status codes for each link
- Summary of successful, redirected, and failed links
- Linting errors if found

Example output:

```bash
Found 5 Markdown files.
Analyzing . or ./folder_name

Total: 25 links to check
[1/25] 200 - GOOD - <https://example.com>
[2/25] 301 - GOOD - <https://google.com>
[3/25] 404 - FAIL - <https://invalid-url.com>

Summary:
Successful: 20
Failed: 2
```

## Use with pre-commit

To run tool as part of a pre-commit workflow, add `.pre-commit-config.yaml` to your project:

```bash
repos:
  - repo: https://github.com/eugenescodes/md-check
    rev: main
    hooks:
      - id: md-check
```

## Use with action

- create a workflow file in the target repository:

```bash
mkdir -p .github/workflows
touch .github/workflows/md-check.yml
```

- add below code to `.github/workflows/md-check.yml`

```bash
name: Check Markdown

on:
  push:
    paths:
      - '**.md'
  pull_request:
    paths:
      - '**.md'
  workflow_dispatch:

jobs:
  markdown-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Check Markdown
        uses: eugenescodes/md-check@main

```

## Development

Project Structure

```bash
src/
├── main.rs          # Main application entry point
├── link_checker.rs  # Link checking functionality
└── linter.rs        # Markdown linting rules


# Run anc check test markdown file

cargo run -- test-cases.md

# Build in debug mode

cargo build

# Run tests

cargo test

# Build in release mode

cargo build --release
```

## Acknowledgments

During the development of this project, I was inspired by the following excellent projects:

- [markdownlint](https://github.com/DavidAnson/markdownlint) - a tool for checking the style and formatting of Markdown files
- [lychee](https://github.com/lycheeverse/lychee/) - a fast link-checking tool written in Rust

## License

This project is licensed under the GNU General Public License v3.0
