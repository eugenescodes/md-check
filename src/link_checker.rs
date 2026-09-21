use colored::*;
use futures::stream::{self, StreamExt};
use reqwest::{Client, StatusCode, redirect::Policy};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct LinkInfo {
    pub url: String,
    pub file_path: PathBuf,
}

#[derive(Debug)]
pub struct CheckResult {
    pub link: LinkInfo,
    /// Final HTTP status code. `None` means the request itself failed
    /// (DNS, connection or timeout error) and no HTTP status was received.
    pub status: Option<StatusCode>,
    pub error_message: Option<String>,
}

const MAX_RETRIES: u32 = 3;
const INITIAL_BACKOFF: Duration = Duration::from_millis(500);

/// Asynchronously checks a list of extracted links by making HTTP requests.
///
/// This function uses a concurrent stream to verify the status of each URL.
///
/// # Examples
///
/// ```no_run
/// # #[tokio::main]
/// # async fn main() {
/// use std::path::PathBuf;
/// use md_check::link_checker::{check_links, LinkInfo};
///
/// let links = vec![LinkInfo {
///     url: "https://www.rust-lang.org".to_string(),
///     file_path: PathBuf::from("test.md"),
/// }];
///
/// // This will perform actual network requests
/// let results = check_links(links).await;
///
/// assert_eq!(results.len(), 1);
/// assert!(results[0].status.is_some());
/// # }
/// ```
pub async fn check_links(links: Vec<LinkInfo>) -> Vec<CheckResult> {
    let is_github_actions = std::env::var("GITHUB_ACTIONS").is_ok_and(|value| value == "true");

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

    if is_github_actions {
        println!("::group::Checking links");
    }

    let counter = Arc::new(AtomicUsize::new(0));

    let results = stream::iter(links)
        .map(|link| {
            let client = client.clone();
            let counter = Arc::clone(&counter);
            async move {
                let result = check_single_link(&client, link.clone()).await;
                let current = counter.fetch_add(1, Ordering::Relaxed) + 1;

                let status_str = match result.status {
                    Some(code) => match code.as_u16() {
                        200..=299 => code.to_string().green(),
                        300..=399 => code.to_string().yellow(),
                        _ => code.to_string().red(),
                    },
                    None => "network error".to_string().red().bold(),
                };

                if is_github_actions {
                    let status_text = result
                        .status
                        .map(|code| code.to_string())
                        .unwrap_or_else(|| "network error".to_string());

                    if result.status.as_ref().is_some_and(|code| code.is_success()) {
                        println!(
                            "::debug::Link {} status: {} (success)",
                            link.url, status_text
                        );
                    } else {
                        println!(
                            "::error file={}::Link {} failed: {}{}",
                            link.file_path.display(),
                            link.url,
                            status_text,
                            result
                                .error_message
                                .as_ref()
                                .map(|m| format!(" - {m}"))
                                .unwrap_or_default()
                        );
                    }
                } else {
                    println!(
                        "[{}/{}] {} - {} - {}",
                        current,
                        total_links,
                        status_str,
                        if result.status.as_ref().is_some_and(|code| code.is_success()) {
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
    let successful = results
        .iter()
        .filter(|r| r.status.as_ref().is_some_and(|s| s.is_success()))
        .count();
    let redirects = results
        .iter()
        .filter(|r| r.status.as_ref().is_some_and(|s| s.is_redirection()))
        .count();
    let failed = results
        .iter()
        .filter(|r| match r.status {
            Some(s) => s.is_client_error() || s.is_server_error(),
            None => true, // network errors count as failures
        })
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
    let initial_url = link.url.clone();
    let mut backoff = INITIAL_BACKOFF;
    let mut retries = MAX_RETRIES;

    loop {
        match client.get(&link.url).send().await {
            Ok(response) => {
                let status = response.status();
                return CheckResult {
                    link: LinkInfo {
                        url: initial_url,
                        file_path: link.file_path,
                    },
                    status: Some(status),
                    error_message: if status.is_success() {
                        None
                    } else {
                        Some(format!("HTTP {status}"))
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
                        status: None,
                        error_message: Some(format!("Request failed: {e}")),
                    };
                }
                // Exponential backoff: 500ms, 1s, 2s, ...
                tokio::time::sleep(backoff).await;
                backoff = backoff.mul_f32(2.0);
            }
        }
    }
}

/// Formats the results of link checks into human-readable error messages.
///
/// It filters out successful requests and returns formatted strings for
/// broken links, client errors, or network failures.
///
/// # Examples
///
/// ```
/// use std::path::PathBuf;
/// use reqwest::StatusCode;
/// use md_check::link_checker::{format_check_results, CheckResult, LinkInfo};
///
/// let results = vec![
///     CheckResult {
///         link: LinkInfo {
///             url: "https://invalid.domain.xyz".to_string(),
///             file_path: PathBuf::from("doc.md"),
///         },
///         status: Some(StatusCode::NOT_FOUND),
///         error_message: Some("Not Found".to_string()),
///     }
/// ];
///
/// let formatted = format_check_results(&results);
///
/// assert_eq!(formatted.len(), 1);
/// assert!(formatted[0].contains("https://invalid.domain.xyz"));
/// ```
pub fn format_check_results(results: &[CheckResult]) -> Vec<String> {
    results
        .iter()
        .filter(|r| !r.status.as_ref().is_some_and(|s| s.is_success()))
        .map(|r| {
            let status_str = match r.status {
                Some(code) if code.is_redirection() => code.to_string().yellow(),
                Some(code) => code.to_string().red(),
                None => "network error".to_string().red().bold(),
            };

            format!(
                "- {} (Status: {}{}) [in file {}]",
                r.link.url,
                status_str,
                r.error_message
                    .as_ref()
                    .map(|msg| format!(" - {msg}"))
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
            .with_header("Location", &format!("{server_url}/final"))
            .create_async()
            .await;

        // Mock the final destination after redirect
        let mock_final = server
            .mock("GET", "/final")
            .with_status(200) // OK
            .create_async()
            .await;

        let link_info = LinkInfo {
            url: format!("{server_url}/redirect"),
            file_path: PathBuf::from("test.md"),
        };

        let results = check_links(vec![link_info]).await;

        assert_eq!(results.len(), 1);
        let result = &results[0];

        assert_eq!(result.link.url, format!("{server_url}/redirect"));
        assert!(result.status.is_some_and(|s| s.is_success()));
        assert!(result.status.is_some_and(|s| s.as_u16() == 200));

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
            .with_header("Location", &format!("{server_url}/notfound"))
            .create_async()
            .await;

        // Mock the final destination to return an error
        let mock_final = server
            .mock("GET", "/notfound")
            .with_status(404) // Not Found
            .create_async()
            .await;

        let link_info = LinkInfo {
            url: format!("{server_url}/redirect-error"),
            file_path: PathBuf::from("test.md"),
        };

        let results = check_links(vec![link_info]).await;

        assert_eq!(results.len(), 1);
        let result = &results[0];

        assert_eq!(result.link.url, format!("{server_url}/redirect-error"));
        assert!(result.status.is_some_and(|s| s.is_client_error()));
        assert!(result.status.is_some_and(|s| s.as_u16() == 404));

        // Verify mocks were called
        mock_redirect.assert_async().await;
        mock_final.assert_async().await;
    }

    #[tokio::test]
    async fn test_network_error_has_no_status() {
        // Port 1 on localhost is virtually guaranteed to be closed
        let link_info = LinkInfo {
            url: "http://127.0.0.1:1/ping".to_string(),
            file_path: PathBuf::from("test.md"),
        };

        let results = check_links(vec![link_info]).await;

        assert_eq!(results.len(), 1);
        assert!(results[0].status.is_none());
        let message = results[0].error_message.as_deref().unwrap();
        assert!(message.contains("Request failed"));
    }
}
