//! CLI argument definitions using clap derive
//!
//! This module defines all command-line arguments and subcommands for repomix.

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// Repomix - Pack your repository into a single AI-friendly file
#[derive(Parser, Debug, Clone)]
#[command(name = "repomix")]
#[command(author = "Repomix Contributors")]
#[command(version)]
#[command(about = "Pack repository contents to single file for AI consumption")]
#[command(long_about = "Repomix is a tool that packs your entire repository into a single, \
    AI-friendly file. It's perfect for when you need to feed your codebase to Large Language \
    Models (LLMs) or other AI tools that work better with consolidated input.")]
pub struct Args {
    /// Directories to pack (defaults to current directory)
    #[arg(default_value = ".")]
    pub directories: Vec<PathBuf>,

    /// Subcommand to run
    #[command(subcommand)]
    pub command: Option<Command>,

    // ============== CLI Input/Output Options ==============
    
    /// Enable detailed debug logging
    #[arg(long, conflicts_with = "quiet")]
    pub verbose: bool,

    /// Suppress all console output except errors
    #[arg(long, conflicts_with = "verbose")]
    pub quiet: bool,

    /// Write packed output directly to stdout instead of a file
    #[arg(long, conflicts_with = "output")]
    pub stdout: bool,

    /// Read file paths from stdin, one per line
    #[arg(long)]
    pub stdin: bool,

    /// Copy the generated output to system clipboard
    #[arg(long)]
    pub copy: bool,

    /// Show file tree with token counts
    #[arg(long, value_name = "THRESHOLD")]
    pub token_count_tree: Option<Option<u32>>,

    /// Number of largest files to show in summary (default: 5)
    #[arg(long, default_value = "5")]
    pub top_files_len: usize,

    // ============== Repomix Output Options ==============
    
    /// Output file path (use "-" for stdout)
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,

    /// Output format style
    #[arg(long, value_enum, default_value = "xml")]
    pub style: OutputStyle,

    /// Escape special characters to ensure valid XML/Markdown
    #[arg(long)]
    pub parsable_style: bool,

    /// Extract essential code structure using Tree-sitter parsing
    #[arg(long)]
    pub compress: bool,

    /// Prefix each line with its line number in the output
    #[arg(long = "output-show-line-numbers")]
    pub show_line_numbers: bool,

    /// Omit the file summary section from output
    #[arg(long = "no-file-summary")]
    pub no_file_summary: bool,

    /// Omit the directory tree visualization from output
    #[arg(long = "no-directory-structure")]
    pub no_directory_structure: bool,

    /// Generate metadata only without file contents
    #[arg(long = "no-files")]
    pub no_files: bool,

    /// Strip all code comments before packing
    #[arg(long)]
    pub remove_comments: bool,

    /// Remove blank lines from all files
    #[arg(long)]
    pub remove_empty_lines: bool,

    /// Truncate long base64 data strings
    #[arg(long)]
    pub truncate_base64: bool,

    /// Custom text to include at the beginning of the output
    #[arg(long, value_name = "TEXT")]
    pub header_text: Option<String>,

    /// Path to file containing custom instructions
    #[arg(long, value_name = "PATH")]
    pub instruction_file_path: Option<PathBuf>,

    /// Include folders with no files in directory structure
    #[arg(long)]
    pub include_empty_directories: bool,

    // ============== File Selection Options ==============
    
    /// Include only files matching these glob patterns (comma-separated)
    #[arg(long, value_delimiter = ',', value_name = "PATTERNS")]
    pub include: Vec<String>,

    /// Additional patterns to exclude (comma-separated)
    #[arg(short = 'i', long, value_delimiter = ',', value_name = "PATTERNS")]
    pub ignore: Vec<String>,

    /// Don't use .gitignore rules for filtering files
    #[arg(long = "no-gitignore")]
    pub no_gitignore: bool,

    /// Don't apply built-in ignore patterns
    #[arg(long = "no-default-patterns")]
    pub no_default_patterns: bool,

    // ============== Remote Repository Options ==============
    
    /// Clone and pack a remote repository (GitHub URL or user/repo format)
    #[arg(long, value_name = "URL")]
    pub remote: Option<String>,

    /// Specific branch, tag, or commit to use
    #[arg(long, value_name = "NAME")]
    pub remote_branch: Option<String>,

    // ============== Configuration Options ==============
    
    /// Use custom config file instead of repomix.config.json
    #[arg(short, long, value_name = "PATH")]
    pub config: Option<PathBuf>,

    /// Create a new repomix.config.json file with defaults
    #[arg(long)]
    pub init: bool,

    /// With --init, create config in home directory
    #[arg(long)]
    pub global: bool,

    // ============== Security Options ==============
    
    /// Skip scanning for sensitive data like API keys and passwords
    #[arg(long = "no-security-check")]
    pub no_security_check: bool,

    // ============== Token Count Options ==============
    
    /// Tokenizer model for counting (default: o200k_base)
    #[arg(long, value_name = "ENCODING", default_value = "o200k_base")]
    pub token_count_encoding: String,
}

/// Subcommands for repomix
#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// Clone and process a remote repository
    Remote {
        /// Repository URL or shorthand (e.g., user/repo, github:user/repo)
        url: String,

        /// Specific branch, tag, or commit to use
        #[arg(long, short)]
        branch: Option<String>,

        /// Output format style
        #[arg(long, value_enum, default_value = "xml")]
        style: OutputStyle,

        /// Output file path
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,

        /// Extract essential code structure using Tree-sitter parsing
        #[arg(long)]
        compress: bool,

        /// Include only files matching these glob patterns (comma-separated)
        #[arg(long, value_delimiter = ',', value_name = "PATTERNS")]
        include: Vec<String>,

        /// Additional patterns to exclude (comma-separated)
        #[arg(short = 'i', long, value_delimiter = ',', value_name = "PATTERNS")]
        ignore: Vec<String>,
    },

    /// Initialize a repomix.config.json file
    Init {
        /// Create config in home directory instead of current directory
        #[arg(long)]
        global: bool,
    },
}

/// Output style format
#[derive(ValueEnum, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum OutputStyle {
    /// XML format (default)
    #[default]
    Xml,
    /// Markdown format
    Markdown,
    /// JSON format
    Json,
    /// Plain text format
    Plain,
}

impl OutputStyle {
    /// Get the default output file name for this style
    pub fn default_file_name(&self) -> &'static str {
        match self {
            OutputStyle::Xml => "repomix-output.xml",
            OutputStyle::Markdown => "repomix-output.md",
            OutputStyle::Json => "repomix-output.json",
            OutputStyle::Plain => "repomix-output.txt",
        }
    }

    /// Get the file extension for this style
    pub fn extension(&self) -> &'static str {
        match self {
            OutputStyle::Xml => "xml",
            OutputStyle::Markdown => "md",
            OutputStyle::Json => "json",
            OutputStyle::Plain => "txt",
        }
    }
}

impl std::fmt::Display for OutputStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputStyle::Xml => write!(f, "xml"),
            OutputStyle::Markdown => write!(f, "markdown"),
            OutputStyle::Json => write!(f, "json"),
            OutputStyle::Plain => write!(f, "plain"),
        }
    }
}

impl From<OutputStyle> for String {
    fn from(style: OutputStyle) -> Self {
        style.to_string()
    }
}

impl TryFrom<&str> for OutputStyle {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "xml" => Ok(OutputStyle::Xml),
            "markdown" | "md" => Ok(OutputStyle::Markdown),
            "json" => Ok(OutputStyle::Json),
            "plain" | "txt" | "text" => Ok(OutputStyle::Plain),
            _ => Err(format!("Unknown output style: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_style_default_file_name() {
        assert_eq!(OutputStyle::Xml.default_file_name(), "repomix-output.xml");
        assert_eq!(OutputStyle::Markdown.default_file_name(), "repomix-output.md");
        assert_eq!(OutputStyle::Json.default_file_name(), "repomix-output.json");
        assert_eq!(OutputStyle::Plain.default_file_name(), "repomix-output.txt");
    }

    #[test]
    fn test_output_style_display() {
        assert_eq!(OutputStyle::Xml.to_string(), "xml");
        assert_eq!(OutputStyle::Markdown.to_string(), "markdown");
    }

    #[test]
    fn test_output_style_try_from() {
        assert_eq!(OutputStyle::try_from("xml"), Ok(OutputStyle::Xml));
        assert_eq!(OutputStyle::try_from("markdown"), Ok(OutputStyle::Markdown));
        assert_eq!(OutputStyle::try_from("md"), Ok(OutputStyle::Markdown));
        assert!(OutputStyle::try_from("invalid").is_err());
    }
}
