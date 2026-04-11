//! Repomix library - Pack repository contents for AI consumption
//!
//! This library provides functionality to:
//! - Search and collect files from a directory
//! - Apply gitignore and custom ignore patterns
//! - Compress code using tree-sitter
//! - Generate output in XML, Markdown, JSON, or plain text

pub mod api;
pub mod cli;
pub mod config;
pub mod core;
pub mod remote;
pub mod shared;

// Re-exports for public API
pub use api::{build_config, pack_directory, pack_remote, PackOptions, PackResult};
pub use cli::args::OutputStyle;
pub use config::schema::{MergedConfig, RepomixConfig};
