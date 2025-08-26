use colored::*;
use futures::stream::{self, StreamExt};
use pulldown_cmark::{Event, Parser, Tag};
use reqwest::{Client, StatusCode, redirect::Policy};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use url::Url;

#[derive(Debug, Clone)]
pub struct LinkInfo {
    pub url: String,
    pub file_path: PathBuf,
}

#[derive(Debug)]
pub struct CheckResult {
    pub link: LinkInfo,
    pub status: StatusCode,
    pub error_message: Option<String>,
}

pub fn extract_links(content: &str, file_path: &Path) -> Vec<LinkInfo> {
    let parser = Parser::new(content);
    let mut links = Vec::new();

    for event in parser {
        if let Event::Start(Tag::Link { dest_url, .. }) = event {
            let url_str = dest_url.to_string();
            if (url_str.starts_with("http://") || url_str.starts_with("https://"))
                && Url::parse(&url_str).is_ok()
            {
                links.push(LinkInfo {
                    url: url_str,
                    file_path: file_path.to_path_buf(),
                });
            }
        }
    }
    links
}

static COUNTER: AtomicUsize = AtomicUsize::new(0);

pub async fn check_links(links: Vec<LinkInfo>) -> Vec<CheckResult> {
    let is_github_actions = std::env::var("GITHUB_ACTIONS").is_ok();

    let client = Client::builder()
        .redirect(Policy::limited(10))
        .timeout(Duration::from_secs(30))
        .user_agent(format!(
            "markdown-link-checker/{}",
            env!("CARGO_PKG_VERSION")
        ))
        .build()
        .unwrap_or_default();

    let total_links = links.len();

    if is_github_actions {
        println!("::notice::Found {} links to check", total_links);
        for link in &links {
            println!(
                "::debug::Checking link: {} in {}",
                link.url,
                link.file_path.display()
            );
        }
    }

    println!("\n{} {} links to check", "Total:".bold(), total_links);

    COUNTER.store(0, Ordering::SeqCst);

    let results = stream::iter(links)
        .map(|link| {
            let client = client.clone();
            async move {
                let result = check_single_link(&client, link.clone()).await;
                let current = COUNTER.fetch_add(1, Ordering::SeqCst) + 1;

                let status_str = match result.status.as_u16() {
                    200..=299 => result.status.to_string().green(),
                    300..=399 => result.status.to_string().yellow(),
                    400..=499 => result.status.to_string().red(),
                    _ => result.status.to_string().red().bold(),
                };

                if is_github_actions {
                    if result.status.is_success() {
                        println!(
                            "::debug::Link {} status: {} (success)",
                            link.url, result.status
                        );
                    } else {
                        println!(
                            "::error file={}::Link {} failed with status {}{}",
                            link.file_path.display(),
                            link.url,
                            result.status,
                            result
                                .error_message
                                .as_ref()
                                .map(|m| format!(" - {}", m))
                                .unwrap_or_default()
                        );
                    }
                } else {
                    println!(
                        "[{}/{}] {} - {} - {}",
                        current,
                        total_links,
                        status_str,
                        if result.status.is_success() {
                            "GOOD".green()
                        } else {
                            "FAIL".red()
                        },
                        link.url
                    );
                }
                result
            }
        })
        .buffer_unordered(10)
        .collect::<Vec<_>>()
        .await;

    if is_github_actions {
        println!("::endgroup::");
    }

    // Print summary
    let successful = results.iter().filter(|r| r.status.is_success()).count();
    let redirects = results.iter().filter(|r| r.status.is_redirection()).count();
    let failed = results
        .iter()
        .filter(|r| r.status.is_client_error() || r.status.is_server_error())
        .count();

    if is_github_actions {
        println!("::group::Summary");
    }

    println!("\n{}", "Link check completed.".bold());
    println!("\n{}", "Summary:".bold());
    println!("{}: {}", "Successful".green(), successful);
    if redirects > 0 {
        println!("{}: {}", "Redirects".yellow(), redirects);
    }
    if failed > 0 {
        println!("{}: {}", "Failed".red(), failed);
    }

    if is_github_actions {
        println!("::endgroup::");
    }

    results
}

async fn check_single_link(client: &Client, link: LinkInfo) -> CheckResult {
    let mut retries = 3;
    let initial_url = link.url.clone();

    loop {
        match client.get(&link.url).send().await {
            Ok(response) => {
                return CheckResult {
                    link: LinkInfo {
                        url: initial_url,
                        file_path: link.file_path,
                    },
                    status: response.status(),
                    error_message: if response.status().is_success() {
                        None
                    } else {
                        Some(format!("HTTP {}", response.status()))
                    },
                };
            }
            Err(e) => {
                retries -= 1;
                if retries == 0 {
                    return CheckResult {
                        link: LinkInfo {
                            url: initial_url,
                            file_path: link.file_path,
                        },
                        status: StatusCode::INTERNAL_SERVER_ERROR,
                        error_message: Some(format!("Request failed: {}", e)),
                    };
                }
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

pub fn format_check_results(results: &[CheckResult]) -> Vec<String> {
    results
        .iter()
        .filter(|r| !r.status.is_success())
        .map(|r| {
            let status_color = if r.status.is_redirection() {
                r.status.to_string().yellow()
            } else {
                r.status.to_string().red()
            };

            format!(
                "- {} (Status: {}{}) [in file {}]",
                r.link.url,
                status_color,
                r.error_message
                    .as_ref()
                    .map(|msg| format!(" - {}", msg))
                    .unwrap_or_default(),
                r.link.file_path.display()
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    

    #[tokio::test]
    async fn test_redirect_handling() {
        let mut server = mockito::Server::new_async().await;
        let server_url = server.url();

        // Mock the initial request to redirect
        let mock_redirect = server
            .mock("GET", "/redirect")
            .with_status(301) // Permanent Redirect
            .with_header("Location", &format!("{}/final", server_url))
            .create_async()
            .await;

        // Mock the final destination after redirect
        let mock_final = server
            .mock("GET", "/final")
            .with_status(200) // OK
            .create_async()
            .await;

        let link_info = LinkInfo {
            url: format!("{}/redirect", server_url),
            file_path: PathBuf::from("test.md"),
        };

        let results = check_links(vec![link_info]).await;

        assert_eq!(results.len(), 1);
        let result = &results[0];

        assert_eq!(result.link.url, format!("{}/redirect", server_url));
        assert!(result.status.is_success()); // Check if the final status was success
        assert_eq!(result.status.as_u16(), 200); // Specifically check for 200 OK

        // Verify mocks were called
        mock_redirect.assert_async().await;
        mock_final.assert_async().await;
    }

    #[tokio::test]
    async fn test_redirect_to_error() {
        let mut server = mockito::Server::new_async().await;
        let server_url = server.url();

        // Mock the initial request to redirect
        let mock_redirect = server
            .mock("GET", "/redirect-error")
            .with_status(302) // Found (Temporary Redirect)
            .with_header("Location", &format!("{}/notfound", server_url))
            .create_async()
            .await;

        // Mock the final destination to return an error
        let mock_final = server
            .mock("GET", "/notfound")
            .with_status(404) // Not Found
            .create_async()
            .await;

        let link_info = LinkInfo {
            url: format!("{}/redirect-error", server_url),
            file_path: PathBuf::from("test.md"),
        };

        let results = check_links(vec![link_info]).await;

        assert_eq!(results.len(), 1);
        let result = &results[0];

        assert_eq!(result.link.url, format!("{}/redirect-error", server_url));
        assert!(result.status.is_client_error()); // Check if the final status was a client error
        assert_eq!(result.status.as_u16(), 404); // Specifically check for 404

        // Verify mocks were called
        mock_redirect.assert_async().await;
        mock_final.assert_async().await;
    }
}
