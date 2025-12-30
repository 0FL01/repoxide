//! Configuration schema definitions
//!
//! This module defines the configuration structures that mirror the TypeScript version.

use serde::{Deserialize, Serialize};

/// Main configuration structure (loaded from file)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub struct RepomixConfig {
    /// Schema URL for IDE support
    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,

    /// Input configuration
    #[serde(default)]
    pub input: InputConfig,

    /// Output configuration
    #[serde(default)]
    pub output: OutputConfig,

    /// Include patterns
    #[serde(default)]
    pub include: Vec<String>,

    /// Ignore configuration
    #[serde(default)]
    pub ignore: IgnoreConfig,

    /// Security configuration
    #[serde(default)]
    pub security: SecurityConfig,

    /// Token count configuration
    #[serde(default)]
    pub token_count: TokenCountConfig,
}


/// Input configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputConfig {
    /// Maximum file size in bytes (default: 50MB)
    #[serde(default = "default_max_file_size")]
    pub max_file_size: usize,
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            max_file_size: default_max_file_size(),
        }
    }
}

fn default_max_file_size() -> usize {
    50 * 1024 * 1024 // 50MB
}

/// Output configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputConfig {
    /// Output file path
    #[serde(default = "default_output_file")]
    pub file_path: String,

    /// Output style (xml, markdown, json, plain)
    #[serde(default = "default_style")]
    pub style: String,

    /// Use parsable style (escape special characters)
    #[serde(default)]
    pub parsable_style: bool,

    /// Show line numbers in output
    #[serde(default)]
    pub show_line_numbers: bool,

    /// Remove comments from code
    #[serde(default)]
    pub remove_comments: bool,

    /// Remove empty lines from code
    #[serde(default)]
    pub remove_empty_lines: bool,

    /// Enable tree-sitter compression
    #[serde(default)]
    pub compress: bool,

    /// Header text to include at the beginning
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header_text: Option<String>,

    /// Path to instruction file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instruction_file_path: Option<String>,

    /// Copy output to clipboard
    #[serde(default)]
    pub copy_to_clipboard: bool,

    /// Include file summary section
    #[serde(default = "default_true")]
    pub file_summary: bool,

    /// Include directory structure section
    #[serde(default = "default_true")]
    pub directory_structure: bool,

    /// Include file contents section
    #[serde(default = "default_true")]
    pub files: bool,

    /// Number of top files to show in summary
    #[serde(default = "default_top_files_length")]
    pub top_files_length: usize,

    /// Truncate base64 data
    #[serde(default)]
    pub truncate_base64: bool,

    /// Include empty directories in tree
    #[serde(default)]
    pub include_empty_directories: bool,

    /// Include full directory structure
    #[serde(default)]
    pub include_full_directory_structure: bool,

    /// Git-related output options
    #[serde(default)]
    pub git: GitOutputConfig,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            file_path: default_output_file(),
            style: default_style(),
            parsable_style: false,
            show_line_numbers: false,
            remove_comments: false,
            remove_empty_lines: false,
            compress: false,
            header_text: None,
            instruction_file_path: None,
            copy_to_clipboard: false,
            file_summary: true,
            directory_structure: true,
            files: true,
            top_files_length: default_top_files_length(),
            truncate_base64: false,
            include_empty_directories: false,
            include_full_directory_structure: false,
            git: GitOutputConfig::default(),
        }
    }
}

/// Git-related output configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitOutputConfig {
    /// Sort files by git change frequency
    #[serde(default = "default_true")]
    pub sort_by_changes: bool,

    /// Maximum commits to analyze for sorting
    #[serde(default = "default_sort_by_changes_max_commits")]
    pub sort_by_changes_max_commits: usize,

    /// Include git diffs in output
    #[serde(default)]
    pub include_diffs: bool,

    /// Include git logs in output
    #[serde(default)]
    pub include_logs: bool,

    /// Number of log entries to include
    #[serde(default = "default_include_logs_count")]
    pub include_logs_count: usize,
}

impl Default for GitOutputConfig {
    fn default() -> Self {
        Self {
            sort_by_changes: true,
            sort_by_changes_max_commits: default_sort_by_changes_max_commits(),
            include_diffs: false,
            include_logs: false,
            include_logs_count: default_include_logs_count(),
        }
    }
}

/// Ignore configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IgnoreConfig {
    /// Use .gitignore rules
    #[serde(default = "default_true")]
    pub use_gitignore: bool,

    /// Use .ignore files
    #[serde(default = "default_true")]
    pub use_dot_ignore: bool,

    /// Use default ignore patterns
    #[serde(default = "default_true")]
    pub use_default_patterns: bool,

    /// Custom ignore patterns
    #[serde(default)]
    pub custom_patterns: Vec<String>,
}

impl Default for IgnoreConfig {
    fn default() -> Self {
        Self {
            use_gitignore: true,
            use_dot_ignore: true,
            use_default_patterns: true,
            custom_patterns: Vec::new(),
        }
    }
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityConfig {
    /// Enable secret/sensitive data detection
    #[serde(default = "default_true")]
    pub enable_security_check: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_security_check: true,
        }
    }
}

/// Token count configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenCountConfig {
    /// Tokenizer encoding to use
    #[serde(default = "default_encoding")]
    pub encoding: String,
}

impl Default for TokenCountConfig {
    fn default() -> Self {
        Self {
            encoding: default_encoding(),
        }
    }
}

// ============== Merged Configuration ==============

/// Merged configuration (defaults + file + CLI)
/// This is the final configuration used by the application
#[derive(Debug, Clone)]
#[derive(Default)]
pub struct MergedConfig {

    pub input: InputConfig,
    pub output: OutputConfig,
    pub include: Vec<String>,
    pub ignore: IgnoreConfig,
    pub security: SecurityConfig,

}




// ============== Default Value Functions ==============

fn default_output_file() -> String {
    "repomix-output.xml".to_string()
}

fn default_style() -> String {
    "xml".to_string()
}

fn default_encoding() -> String {
    "o200k_base".to_string()
}

fn default_true() -> bool {
    true
}

fn default_top_files_length() -> usize {
    5
}

fn default_sort_by_changes_max_commits() -> usize {
    100
}

fn default_include_logs_count() -> usize {
    50
}

// ============== Output Style Helpers ==============



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = RepomixConfig::default();
        assert_eq!(config.output.file_path, "repomix-output.xml");
        assert_eq!(config.output.style, "xml");
        assert!(config.ignore.use_gitignore);
        assert!(config.security.enable_security_check);
    }



    #[test]
    fn test_serialize_config() {
        let config = RepomixConfig::default();
        let json = serde_json::to_string_pretty(&config).unwrap();
        assert!(json.contains("filePath"));
        assert!(json.contains("repomix-output.xml"));
    }

    #[test]
    fn test_deserialize_config() {
        let json = r#"{
            "output": {
                "filePath": "custom-output.xml",
                "style": "markdown"
            },
            "include": ["src/**/*.rs"],
            "ignore": {
                "customPatterns": ["target/**"]
            }
        }"#;
        
        let config: RepomixConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.output.file_path, "custom-output.xml");
        assert_eq!(config.output.style, "markdown");
        assert_eq!(config.include, vec!["src/**/*.rs"]);
        assert_eq!(config.ignore.custom_patterns, vec!["target/**"]);
    }
}
