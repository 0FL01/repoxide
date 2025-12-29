//! Git repository cloning
//!
//! Handles cloning remote repositories to a temporary directory
//! using `git clone --depth 1` for efficiency.

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

use super::parse::RemoteInfo;

/// Result of a clone operation
pub struct CloneResult {
    /// Path to the cloned repository
    pub path: PathBuf,
    /// Temporary directory handle (keeps the directory alive)
    _temp_dir: TempDir,
}

impl CloneResult {
    /// Get the path to the cloned repository
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

/// Check if git is installed and available in PATH
pub fn is_git_installed() -> bool {
    Command::new("git")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Clone a repository to a temporary directory
///
/// # Arguments
/// * `info` - Parsed remote repository information
/// * `branch` - Optional branch/tag/commit to checkout
///
/// # Returns
/// A `CloneResult` containing the path to the cloned repository.
/// The temporary directory is automatically cleaned up when `CloneResult` is dropped.
pub fn clone_repository(info: &RemoteInfo, branch: Option<&str>) -> Result<CloneResult> {
    // Check if git is installed
    if !is_git_installed() {
        anyhow::bail!("Git is not installed or not in the system PATH");
    }
    
    // Create temporary directory
    let temp_dir = TempDir::with_prefix("repomix-")
        .context("Failed to create temporary directory")?;
    
    let target_path = temp_dir.path().to_path_buf();
    
    // Build git clone command
    let mut cmd = Command::new("git");
    cmd.arg("clone")
        .arg("--depth").arg("1");  // Shallow clone for speed
    
    // Add branch if specified
    let effective_branch = branch.or(info.branch.as_deref());
    if let Some(b) = effective_branch {
        cmd.arg("--branch").arg(b);
    }
    
    // Add URL and target directory
    cmd.arg(&info.url)
        .arg(&target_path);
    
    // Execute clone
    let output = cmd.output()
        .context("Failed to execute git clone command")?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Git clone failed: {}", stderr.trim());
    }
    
    Ok(CloneResult {
        path: target_path,
        _temp_dir: temp_dir,
    })
}

/// Clone a repository from a URL string
///
/// Convenience function that parses the URL and clones in one step.
///
/// # Arguments
/// * `url` - Remote URL or shorthand (e.g., "user/repo")
/// * `branch` - Optional branch to checkout
pub fn clone_from_url(url: &str, branch: Option<&str>) -> Result<CloneResult> {
    use super::parse::parse_remote_url;
    
    let info = parse_remote_url(url)
        .context("Invalid remote repository URL or shorthand (owner/repo)")?;
    
    clone_repository(&info, branch)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_installed() {
        // This test just verifies the function doesn't panic
        // The result depends on whether git is installed on the system
        let _ = is_git_installed();
    }

    #[test]
    fn test_clone_invalid_url() {
        let result = clone_from_url("not-a-valid-url", None);
        assert!(result.is_err());
    }

    // Note: Integration tests that actually clone would be in tests/remote.rs
    // and marked with #[ignore] to avoid network calls in CI
}
