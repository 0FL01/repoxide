//! Remote URL parsing
//!
//! Supports multiple URL formats:
//! - `https://github.com/user/repo`
//! - `https://github.com/user/repo.git`
//! - `github:user/repo`
//! - `user/repo` (shorthand for GitHub)
//! - `https://github.com/user/repo/tree/branch` (with branch)
//! - Azure DevOps URLs

use std::fmt;

/// Parsed remote repository information
#[derive(Debug, Clone, PartialEq)]
pub struct RemoteInfo {
    /// Full repository URL for cloning
    pub url: String,
    /// Repository owner/organization
    pub owner: String,
    /// Repository name
    pub repo: String,
    /// Optional branch/tag/commit reference
    pub branch: Option<String>,
}

impl fmt::Display for RemoteInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.owner, self.repo)
    }
}

/// Valid GitHub shorthand pattern: owner/repo
/// Pattern: [a-zA-Z0-9](?:[a-zA-Z0-9._-]*[a-zA-Z0-9])?
fn is_valid_name(name: &str) -> bool {
    if name.is_empty() || name.len() > 100 {
        return false;
    }

    let chars: Vec<char> = name.chars().collect();

    // First and last characters must be alphanumeric
    if !chars[0].is_ascii_alphanumeric() {
        return false;
    }
    if chars.len() > 1 && !chars[chars.len() - 1].is_ascii_alphanumeric() {
        return false;
    }

    // Middle characters can be alphanumeric, '.', '_', or '-'
    for c in &chars[1..chars.len().saturating_sub(1)] {
        if !c.is_ascii_alphanumeric() && *c != '.' && *c != '_' && *c != '-' {
            return false;
        }
    }

    true
}

/// Check if a string is a valid GitHub shorthand (owner/repo)
pub fn is_valid_shorthand(remote_value: &str) -> bool {
    let parts: Vec<&str> = remote_value.split('/').collect();
    if parts.len() != 2 {
        return false;
    }
    is_valid_name(parts[0]) && is_valid_name(parts[1])
}

/// Check if URL is an Azure DevOps repository
fn is_azure_devops_url(remote_value: &str) -> bool {
    // Handle SSH URLs (e.g., git@ssh.dev.azure.com:v3/org/project/repo)
    if remote_value.starts_with("git@ssh.dev.azure.com:") {
        return true;
    }

    // Handle HTTP(S) URLs
    if let Ok(url) = url::Url::parse(remote_value) {
        let hostname = url.host_str().unwrap_or("").to_lowercase();

        // Check for exact Azure DevOps hostnames
        if hostname == "dev.azure.com" || hostname == "ssh.dev.azure.com" {
            return true;
        }

        // Check for legacy Visual Studio Team Services (*.visualstudio.com)
        if hostname.ends_with(".visualstudio.com") {
            return true;
        }
    }

    false
}

/// Parse a remote URL or shorthand into RemoteInfo
///
/// # Supported formats
/// - `user/repo` → `https://github.com/user/repo.git`
/// - `github:user/repo` → `https://github.com/user/repo.git`
/// - `https://github.com/user/repo` → as-is with .git
/// - `https://github.com/user/repo.git` → as-is
/// - `https://github.com/user/repo/tree/branch` → extracts branch
/// - `git@github.com:user/repo.git` → SSH format
/// - Azure DevOps URLs → as-is
pub fn parse_remote_url(input: &str) -> Option<RemoteInfo> {
    let input = input.trim();

    if input.is_empty() {
        return None;
    }

    // Handle GitHub shorthand: user/repo
    if is_valid_shorthand(input) {
        let parts: Vec<&str> = input.split('/').collect();
        return Some(RemoteInfo {
            url: format!("https://github.com/{}/{}.git", parts[0], parts[1]),
            owner: parts[0].to_string(),
            repo: parts[1].to_string(),
            branch: None,
        });
    }

    // Handle github:user/repo format
    if let Some(rest) = input.strip_prefix("github:") {
        if is_valid_shorthand(rest) {
            let parts: Vec<&str> = rest.split('/').collect();
            return Some(RemoteInfo {
                url: format!("https://github.com/{}/{}.git", parts[0], parts[1]),
                owner: parts[0].to_string(),
                repo: parts[1].to_string(),
                branch: None,
            });
        }
    }

    // Handle Azure DevOps URLs (pass through without parsing)
    if is_azure_devops_url(input) {
        // For Azure DevOps, we can't easily extract owner/repo, so use placeholders
        return Some(RemoteInfo {
            url: input.to_string(),
            owner: "azure".to_string(),
            repo: "repo".to_string(),
            branch: None,
        });
    }

    // Handle SSH format: git@github.com:user/repo.git
    if input.starts_with("git@") {
        // Parse git@host:path format
        if let Some(colon_pos) = input.find(':') {
            let _host = &input[4..colon_pos];
            let path = &input[colon_pos + 1..];

            // Remove .git suffix if present
            let path = path.strip_suffix(".git").unwrap_or(path);

            let parts: Vec<&str> = path.split('/').collect();
            if parts.len() >= 2 {
                let owner = parts[0].to_string();
                let repo = parts[1].to_string();

                let url = if input.ends_with(".git") {
                    input.to_string()
                } else {
                    format!("{}.git", input)
                };

                return Some(RemoteInfo {
                    url,
                    owner,
                    repo,
                    branch: None,
                });
            }
        }
        return None;
    }

    // Handle HTTPS URLs
    if input.starts_with("https://") || input.starts_with("http://") {
        if let Ok(url) = url::Url::parse(input) {
            let hostname = url.host_str().unwrap_or("").to_lowercase();

            // Only handle github.com URLs for now
            if hostname == "github.com" || hostname == "www.github.com" {
                let path_segments: Vec<&str> =
                    url.path().split('/').filter(|s| !s.is_empty()).collect();

                if path_segments.len() >= 2 {
                    let owner = path_segments[0].to_string();
                    let repo = path_segments[1].trim_end_matches(".git").to_string();

                    // Extract branch from /tree/branch or /commit/sha patterns
                    let branch = if path_segments.len() >= 4 {
                        let ref_type = path_segments[2];
                        if ref_type == "tree" || ref_type == "commit" {
                            Some(path_segments[3..].join("/"))
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    // Build clean clone URL
                    let clone_url = format!("https://github.com/{}/{}.git", owner, repo);

                    return Some(RemoteInfo {
                        url: clone_url,
                        owner,
                        repo,
                        branch,
                    });
                }
            } else {
                // For non-GitHub URLs, try to extract owner/repo from path
                let path_segments: Vec<&str> =
                    url.path().split('/').filter(|s| !s.is_empty()).collect();

                if path_segments.len() >= 2 {
                    let owner = path_segments[0].to_string();
                    let repo = path_segments[1].trim_end_matches(".git").to_string();

                    let clone_url = if input.ends_with(".git") {
                        input.to_string()
                    } else {
                        format!("{}.git", input.trim_end_matches('/'))
                    };

                    return Some(RemoteInfo {
                        url: clone_url,
                        owner,
                        repo,
                        branch: None,
                    });
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_shorthand() {
        assert!(is_valid_shorthand("user/repo"));
        assert!(is_valid_shorthand("yamadashy/repoxide"));
        assert!(is_valid_shorthand("my-org/my-repo"));
        assert!(is_valid_shorthand("user123/repo456"));
        assert!(is_valid_shorthand("org.name/repo.name"));

        assert!(!is_valid_shorthand("user"));
        assert!(!is_valid_shorthand("user/"));
        assert!(!is_valid_shorthand("/repo"));
        assert!(!is_valid_shorthand("user/repo/extra"));
        assert!(!is_valid_shorthand("-user/repo")); // starts with -
        assert!(!is_valid_shorthand("user/repo-")); // ends with -
    }

    #[test]
    fn test_parse_shorthand() {
        let info = parse_remote_url("yamadashy/repoxide").unwrap();
        assert_eq!(info.owner, "yamadashy");
        assert_eq!(info.repo, "repoxide");
        assert_eq!(info.url, "https://github.com/yamadashy/repoxide.git");
        assert_eq!(info.branch, None);
    }

    #[test]
    fn test_parse_github_prefix() {
        let info = parse_remote_url("github:user/repo").unwrap();
        assert_eq!(info.owner, "user");
        assert_eq!(info.repo, "repo");
        assert_eq!(info.url, "https://github.com/user/repo.git");
    }

    #[test]
    fn test_parse_https_url() {
        let info = parse_remote_url("https://github.com/user/repo").unwrap();
        assert_eq!(info.owner, "user");
        assert_eq!(info.repo, "repo");
        assert_eq!(info.url, "https://github.com/user/repo.git");

        let info = parse_remote_url("https://github.com/user/repo.git").unwrap();
        assert_eq!(info.owner, "user");
        assert_eq!(info.repo, "repo");
        assert_eq!(info.url, "https://github.com/user/repo.git");
    }

    #[test]
    fn test_parse_https_with_branch() {
        let info = parse_remote_url("https://github.com/user/repo/tree/main").unwrap();
        assert_eq!(info.owner, "user");
        assert_eq!(info.repo, "repo");
        assert_eq!(info.branch, Some("main".to_string()));

        let info = parse_remote_url("https://github.com/user/repo/tree/feature/my-branch").unwrap();
        assert_eq!(info.branch, Some("feature/my-branch".to_string()));
    }

    #[test]
    fn test_parse_ssh_url() {
        let info = parse_remote_url("git@github.com:user/repo.git").unwrap();
        assert_eq!(info.owner, "user");
        assert_eq!(info.repo, "repo");
    }

    #[test]
    fn test_azure_devops_detection() {
        assert!(is_azure_devops_url(
            "https://dev.azure.com/org/project/_git/repo"
        ));
        assert!(is_azure_devops_url(
            "git@ssh.dev.azure.com:v3/org/project/repo"
        ));
        assert!(is_azure_devops_url(
            "https://myorg.visualstudio.com/project/_git/repo"
        ));

        assert!(!is_azure_devops_url("https://github.com/user/repo"));
    }

    #[test]
    fn test_invalid_inputs() {
        assert!(parse_remote_url("").is_none());
        assert!(parse_remote_url("   ").is_none());
        assert!(parse_remote_url("not-a-valid-url").is_none());
    }
}
