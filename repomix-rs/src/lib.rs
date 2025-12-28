//! Repomix library - Pack repository contents for AI consumption
//!
//! This library provides functionality to:
//! - Search and collect files from a directory
//! - Apply gitignore and custom ignore patterns
//! - Compress code using tree-sitter
//! - Generate output in XML, Markdown, JSON, or plain text

pub mod cli;
pub mod config;
pub mod core;
pub mod remote;
pub mod shared;

// Re-exports for public API
pub use config::schema::RepomixConfig;
pub use core::packager::pack;
