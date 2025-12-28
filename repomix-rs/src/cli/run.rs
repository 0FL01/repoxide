//! CLI execution logic
//!
//! This module handles parsing CLI arguments and dispatching to the appropriate action.

use anyhow::{Context, Result};
use clap::Parser;
use colored::Colorize;
use std::env;
use std::path::PathBuf;

use super::args::{Args, Command, OutputStyle};
use crate::config::{load_config, MergedConfig, RepomixConfig};

/// Log level for CLI output
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Silent,
    Info,
    Debug,
}

/// CLI execution context
#[derive(Debug)]
pub struct CliContext {
    pub cwd: PathBuf,
    pub args: Args,
    pub config: MergedConfig,
    pub log_level: LogLevel,
}

impl CliContext {
    /// Check if we should output to stdout
    pub fn is_stdout_mode(&self) -> bool {
        self.args.stdout || self.args.output.as_ref().map(|p| p.to_string_lossy() == "-").unwrap_or(false)
    }

    /// Get the output file path
    pub fn output_path(&self) -> PathBuf {
        if let Some(ref path) = self.args.output {
            if path.to_string_lossy() != "-" {
                return path.clone();
            }
        }
        PathBuf::from(self.config.output.file_path.clone())
    }

    /// Log a message (respects log level)
    pub fn log(&self, message: &str) {
        if self.log_level != LogLevel::Silent {
            println!("{}", message);
        }
    }

    /// Log a debug message (only in verbose mode)
    pub fn debug(&self, message: &str) {
        if self.log_level == LogLevel::Debug {
            println!("{} {}", "[DEBUG]".dimmed(), message);
        }
    }
}

/// Run the CLI application
pub fn run() -> Result<()> {
    let args = Args::parse();
    let cwd = env::current_dir().context("Failed to get current directory")?;

    // Determine log level
    let log_level = if args.quiet || args.stdout {
        LogLevel::Silent
    } else if args.verbose {
        LogLevel::Debug
    } else {
        LogLevel::Info
    };

    // Print version header (unless in silent mode or stdin mode)
    if log_level != LogLevel::Silent && !args.stdin {
        let version = env!("CARGO_PKG_VERSION");
        println!("{}", format!("\n📦 Repomix v{}\n", version).dimmed());
    }

    // Handle --init flag
    if args.init {
        return run_init_action(&cwd, args.global);
    }

    // Handle subcommands
    if let Some(ref cmd) = args.command {
        match cmd {
            Command::Remote { url, branch } => {
                return run_remote_action(url, branch.clone(), &args);
            }
            Command::Init { global } => {
                return run_init_action(&cwd, *global);
            }
        }
    }

    // Handle --remote flag
    if let Some(ref url) = args.remote {
        return run_remote_action(url, args.remote_branch.clone(), &args);
    }

    // Default action: process local directories
    run_default_action(&cwd, &args)
}

/// Run the default action (process local directories)
fn run_default_action(cwd: &PathBuf, args: &Args) -> Result<()> {
    // Determine log level for context
    let log_level = if args.quiet || args.stdout {
        LogLevel::Silent
    } else if args.verbose {
        LogLevel::Debug
    } else {
        LogLevel::Info
    };

    // Load configuration
    let config_path = args.config.as_ref().map(|p| p.as_path());
    let file_config = load_config(cwd, config_path)?;

    // Merge CLI args with file config
    let merged_config = merge_cli_with_config(args, file_config);

    let ctx = CliContext {
        cwd: cwd.clone(),
        args: args.clone(),
        config: merged_config,
        log_level,
    };

    ctx.debug(&format!("Working directory: {:?}", cwd));
    ctx.debug(&format!("Directories to process: {:?}", args.directories));
    ctx.debug(&format!("Output style: {}", args.style));
    ctx.debug(&format!("Configuration: {:?}", ctx.config));

    // TODO: Phase 3+ will implement actual file processing
    ctx.log(&format!("Processing {} director{}", 
        args.directories.len(),
        if args.directories.len() == 1 { "y" } else { "ies" }
    ));
    
    for dir in &args.directories {
        ctx.log(&format!("  → {}", dir.display()));
    }

    ctx.log(&format!("\nOutput style: {}", args.style.to_string().cyan()));
    ctx.log(&format!("Output file: {}", ctx.output_path().display().to_string().green()));

    if args.compress {
        ctx.log(&format!("Compression: {}", "enabled".green()));
    }

    Ok(())
}

/// Run the init action (create config file)
fn run_init_action(cwd: &PathBuf, global: bool) -> Result<()> {
    use std::fs;

    let config_dir = if global {
        dirs::home_dir()
            .context("Could not find home directory")?
            .join(".config")
            .join("repomix")
    } else {
        cwd.clone()
    };

    // Create directory if needed
    if global {
        fs::create_dir_all(&config_dir)
            .context("Failed to create config directory")?;
    }

    let config_path = config_dir.join("repomix.config.json");

    if config_path.exists() {
        println!("{} Config file already exists at: {}", 
            "⚠".yellow(), 
            config_path.display().to_string().cyan()
        );
        return Ok(());
    }

    let default_config = RepomixConfig::default();
    let config_json = serde_json::to_string_pretty(&default_config)
        .context("Failed to serialize default config")?;

    fs::write(&config_path, config_json)
        .context("Failed to write config file")?;

    println!("{} Created config file at: {}", 
        "✓".green(), 
        config_path.display().to_string().cyan()
    );

    Ok(())
}

/// Run the remote action (clone and process remote repository)
fn run_remote_action(url: &str, branch: Option<String>, _args: &Args) -> Result<()> {
    // TODO: Phase 6 will implement remote repository support
    println!("{} Remote repository support", "→".cyan());
    println!("  URL: {}", url.green());
    if let Some(ref b) = branch {
        println!("  Branch: {}", b.yellow());
    }
    println!("\n{}", "Remote repository processing will be implemented in Phase 6".dimmed());
    Ok(())
}

/// Merge CLI arguments with file configuration
fn merge_cli_with_config(args: &Args, file_config: RepomixConfig) -> MergedConfig {
    use crate::config::schema::*;

    // Start with defaults
    let mut config = MergedConfig::default();

    // Apply file config
    config.output.file_path = file_config.output.file_path;
    config.output.style = file_config.output.style.clone();
    config.output.show_line_numbers = file_config.output.show_line_numbers;
    config.output.remove_comments = file_config.output.remove_comments;
    config.output.remove_empty_lines = file_config.output.remove_empty_lines;
    config.output.compress = file_config.output.compress;
    config.output.header_text = file_config.output.header_text.clone();
    config.output.instruction_file_path = file_config.output.instruction_file_path.clone();
    config.output.copy_to_clipboard = file_config.output.copy_to_clipboard;
    config.output.file_summary = file_config.output.file_summary;
    config.output.directory_structure = file_config.output.directory_structure;
    config.output.files = file_config.output.files;
    config.output.top_files_length = file_config.output.top_files_length;
    config.output.parsable_style = file_config.output.parsable_style;
    config.output.truncate_base64 = file_config.output.truncate_base64;
    config.output.include_empty_directories = file_config.output.include_empty_directories;

    config.include = file_config.include.clone();
    config.ignore = file_config.ignore.clone();
    config.security = file_config.security.clone();
    config.input = file_config.input.clone();

    // Override with CLI args
    if let Some(ref output) = args.output {
        config.output.file_path = output.to_string_lossy().to_string();
    } else {
        // Auto-adjust file path based on style
        config.output.file_path = args.style.default_file_name().to_string();
    }

    // Apply style from args
    config.output.style = args.style.to_string();

    // Apply boolean flags
    if args.show_line_numbers {
        config.output.show_line_numbers = true;
    }
    if args.remove_comments {
        config.output.remove_comments = true;
    }
    if args.remove_empty_lines {
        config.output.remove_empty_lines = true;
    }
    if args.compress {
        config.output.compress = true;
    }
    if args.copy {
        config.output.copy_to_clipboard = true;
    }
    if args.no_file_summary {
        config.output.file_summary = false;
    }
    if args.no_directory_structure {
        config.output.directory_structure = false;
    }
    if args.no_files {
        config.output.files = false;
    }
    if args.parsable_style {
        config.output.parsable_style = true;
    }
    if args.truncate_base64 {
        config.output.truncate_base64 = true;
    }
    if args.include_empty_directories {
        config.output.include_empty_directories = true;
    }

    // Apply header text
    if let Some(ref text) = args.header_text {
        config.output.header_text = Some(text.clone());
    }

    // Apply instruction file path
    if let Some(ref path) = args.instruction_file_path {
        config.output.instruction_file_path = Some(path.to_string_lossy().to_string());
    }

    // Apply top files length
    config.output.top_files_length = args.top_files_len;

    // Merge include patterns
    if !args.include.is_empty() {
        config.include.extend(args.include.clone());
    }

    // Merge ignore patterns
    if !args.ignore.is_empty() {
        config.ignore.custom_patterns.extend(args.ignore.clone());
    }

    // Apply ignore flags
    if args.no_gitignore {
        config.ignore.use_gitignore = false;
    }
    if args.no_default_patterns {
        config.ignore.use_default_patterns = false;
    }

    // Apply security flag
    if args.no_security_check {
        config.security.enable_security_check = false;
    }

    config
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_from_args() {
        // This would need a way to construct Args for testing
        // For now, just verify LogLevel enum works
        assert_ne!(LogLevel::Silent, LogLevel::Info);
        assert_ne!(LogLevel::Info, LogLevel::Debug);
    }
}
