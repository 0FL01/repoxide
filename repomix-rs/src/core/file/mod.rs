//! File handling module - Search, collect, and read files
//!
//! This module provides comprehensive file system operations:
//! - `search`: Find files with glob patterns, gitignore support, and filtering
//! - `collect`: Read files in parallel with encoding detection
//! - `tree`: Generate ASCII directory tree representation

pub mod collect;
pub mod search;
pub mod tree;

// Re-export main types and functions
pub use collect::{
    collect_files, collect_files_with_progress, CollectedFile, CollectResult, FileSkipReason,
    SkippedFile,
};
pub use search::{search_files, FileSearchResult, DEFAULT_IGNORE_PATTERNS};
pub use tree::{count_lines, generate_tree, generate_tree_with_line_counts};
