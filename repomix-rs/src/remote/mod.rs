//! Remote repository handling
//!
//! This module provides functionality for:
//! - Parsing remote repository URLs (GitHub shorthand, HTTPS, SSH, Azure DevOps)
//! - Cloning repositories to temporary directories
//! - Cleaning up temporary files after processing

pub mod clone;
pub mod parse;

pub use clone::{clone_from_url, clone_repository, is_git_installed, CloneResult};
pub use parse::{is_github_repository, is_valid_shorthand, parse_remote_url, RemoteInfo};
