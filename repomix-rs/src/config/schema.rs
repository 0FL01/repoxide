//! Configuration schema definitions

use serde::{Deserialize, Serialize};

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RepomixConfig {
    /// Output configuration
    #[serde(default)]
    pub output: OutputConfig,
    
    /// Include patterns
    #[serde(default)]
    pub include: Vec<String>,
    
    /// Ignore patterns
    #[serde(default)]
    pub ignore: IgnoreConfig,
    
    /// Security configuration
    #[serde(default)]
    pub security: SecurityConfig,
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
    
    /// Show line numbers
    #[serde(default)]
    pub show_line_numbers: bool,
    
    /// Remove comments
    #[serde(default)]
    pub remove_comments: bool,
    
    /// Enable compression
    #[serde(default)]
    pub compress: bool,
    
    /// Header text
    pub header_text: Option<String>,
    
    /// Instruction file path
    pub instruction_file_path: Option<String>,
    
    /// Copy to clipboard
    #[serde(default)]
    pub copy_to_clipboard: bool,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            file_path: default_output_file(),
            style: default_style(),
            show_line_numbers: false,
            remove_comments: false,
            compress: false,
            header_text: None,
            instruction_file_path: None,
            copy_to_clipboard: false,
        }
    }
}

/// Ignore configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct IgnoreConfig {
    /// Use .gitignore
    #[serde(default = "default_true")]
    pub use_gitignore: bool,
    
    /// Use default ignore patterns
    #[serde(default = "default_true")]
    pub use_default_ignore: bool,
    
    /// Custom ignore patterns
    #[serde(default)]
    pub custom_patterns: Vec<String>,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SecurityConfig {
    /// Enable secret detection
    #[serde(default = "default_true")]
    pub enable_security_check: bool,
}

fn default_output_file() -> String {
    "repomix-output.xml".to_string()
}

fn default_style() -> String {
    "xml".to_string()
}

fn default_true() -> bool {
    true
}
