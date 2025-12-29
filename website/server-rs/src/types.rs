//! Request and Response types for API endpoints

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============== Pack API Types ==============

/// Options for pack operation (from multipart form)
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackOptions {
    /// Remove comments from code
    #[serde(default)]
    pub remove_comments: bool,

    /// Remove empty lines from code
    #[serde(default)]
    pub remove_empty_lines: bool,

    /// Show line numbers in output
    #[serde(default)]
    pub show_line_numbers: bool,

    /// Include file summary section
    #[serde(default)]
    #[allow(dead_code)]
    pub file_summary: bool,

    /// Include directory structure section
    #[serde(default)]
    #[allow(dead_code)]
    pub directory_structure: bool,

    /// Include patterns (comma-separated)
    pub include_patterns: Option<String>,

    /// Ignore patterns (comma-separated)
    pub ignore_patterns: Option<String>,

    /// Use parsable output style
    #[serde(default)]
    #[allow(dead_code)]
    pub output_parsable: bool,

    /// Enable tree-sitter compression
    #[serde(default)]
    pub compress: bool,
}

/// Response from pack operation
#[derive(Debug, Serialize)]
pub struct PackResponse {
    /// Generated output content
    pub content: String,

    /// Output format used
    pub format: String,

    /// Metadata about the pack operation
    pub metadata: PackMetadata,
}

/// Metadata about pack operation
#[derive(Debug, Serialize)]
pub struct PackMetadata {
    /// Repository name or source
    pub repository: String,

    /// Timestamp of pack operation
    pub timestamp: String,

    /// Summary statistics (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<PackSummary>,

    /// Top files by token count
    #[serde(rename = "topFiles", skip_serializing_if = "Option::is_none")]
    pub top_files: Option<Vec<TopFile>>,
}

/// Summary statistics for pack operation
#[derive(Debug, Serialize)]
pub struct PackSummary {
    /// Total number of files processed
    #[serde(rename = "totalFiles")]
    pub total_files: usize,

    /// Total character count
    #[serde(rename = "totalCharacters")]
    pub total_characters: usize,

    /// Total token count
    #[serde(rename = "totalTokens")]
    pub total_tokens: usize,
}

/// Top file by token count
#[derive(Debug, Serialize)]
pub struct TopFile {
    /// File path
    pub path: String,

    /// Character count
    #[serde(rename = "charCount")]
    pub char_count: usize,

    /// Token count
    #[serde(rename = "tokenCount")]
    pub token_count: usize,
}

// ============== Upload API Types ==============

/// Request to initialize chunked upload
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitUploadRequest {
    /// Original file name
    pub file_name: String,

    /// Total file size in bytes
    pub file_size: u64,

    /// Total number of chunks
    pub total_chunks: u32,
}

/// Response from upload initialization
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InitUploadResponse {
    /// Unique upload session ID
    pub upload_id: Uuid,

    /// Session expiration time (seconds from now)
    pub expires_in: u64,
}

/// Response from chunk upload
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChunkResponse {
    /// Upload session ID
    pub upload_id: Uuid,

    /// Number of chunks received
    pub chunks_received: usize,

    /// Total number of chunks expected
    pub total_chunks: u32,

    /// Whether all chunks have been received
    pub complete: bool,
}

/// Response from upload status query
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusResponse {
    /// Upload session ID
    pub upload_id: Uuid,

    /// Number of chunks received
    pub chunks_received: usize,

    /// Total number of chunks expected
    pub total_chunks: u32,

    /// Upload progress (0.0 to 1.0)
    pub progress: f64,

    /// Whether all chunks have been received
    pub complete: bool,
}
