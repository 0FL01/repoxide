//! CLI argument definitions using clap derive

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// A tool to pack repository contents to single file for AI consumption
#[derive(Parser, Debug)]
#[command(name = "repomix")]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Directory to pack (default: current directory)
    #[arg(default_value = ".")]
    pub directory: PathBuf,

    /// Subcommand to run
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Output file path
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Output style format
    #[arg(long, value_enum, default_value = "xml")]
    pub style: OutputStyle,

    /// Enable tree-sitter code compression
    #[arg(long)]
    pub compress: bool,

    /// Glob patterns to include (comma-separated or multiple flags)
    #[arg(long, value_delimiter = ',')]
    pub include: Vec<String>,

    /// Glob patterns to ignore (comma-separated or multiple flags)  
    #[arg(long, value_delimiter = ',')]
    pub ignore: Vec<String>,

    /// Remove comments from code
    #[arg(long)]
    pub remove_comments: bool,

    /// Show line numbers in output
    #[arg(long)]
    pub show_line_numbers: bool,

    /// Copy output to clipboard
    #[arg(long)]
    pub copy: bool,

    /// Output to stdout instead of file
    #[arg(long)]
    pub stdout: bool,

    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Clone and process a remote repository
    Remote {
        /// Repository URL or shorthand (e.g., user/repo, github:user/repo)
        url: String,
    },
    /// Initialize a repomix.config.json file
    Init,
}

#[derive(ValueEnum, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum OutputStyle {
    /// XML format
    #[default]
    Xml,
    /// Markdown format
    Markdown,
    /// JSON format
    Json,
    /// Plain text format
    Plain,
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
