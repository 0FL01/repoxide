//! Metrics module - Token counting and statistics
//!
//! Provides token counting using tiktoken-rs with o200k_base encoding
//! (GPT-4o and newer models).

pub mod tokens;

pub use tokens::{count_tokens, FileMetrics, PackMetrics, PackPhaseTimings};
