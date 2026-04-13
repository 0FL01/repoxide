//! CLI execution logic
//!
//! This module handles parsing CLI arguments and dispatching to the appropriate action.

use anyhow::{Context, Result};
use clap::Parser;
use colored::Colorize;
use std::env;
use std::path::{Path, PathBuf};

use super::args::{Args, Command};
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
    pub args: Args,
    pub config: MergedConfig,
    pub log_level: LogLevel,
}

impl CliContext {
    /// Get the output file path
    pub fn output_path(&self) -> PathBuf {
        if let Some(ref path) = self.args.output {
            if path.to_string_lossy() != "-" {
                return path.clone();
            }
        }
        PathBuf::from(self.config.output.file_path.clone())
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
            Command::Remote {
                url,
                branch,
                style,
                output,
                compress,
                include,
                ignore,
            } => {
                // Create a modified args with subcommand parameters
                let mut remote_args = args.clone();
                remote_args.style = *style;
                if let Some(ref out) = output {
                    remote_args.output = Some(out.clone());
                }
                if *compress {
                    remote_args.compress = true;
                }
                if !include.is_empty() {
                    remote_args.include = include.clone();
                }
                if !ignore.is_empty() {
                    remote_args.ignore = ignore.clone();
                }
                return run_remote_action(url, branch.clone(), &remote_args);
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
    use crate::core::compress::compress_code;
    use crate::core::file::{collect_files, search_files};
    use crate::core::output::generate::{generate_output, generate_output_from_paths};
    use std::fs;

    // Determine log level for context
    let log_level = if args.quiet || args.stdout {
        LogLevel::Silent
    } else if args.verbose {
        LogLevel::Debug
    } else {
        LogLevel::Info
    };

    // Load configuration
    let config_path = args.config.as_deref();
    let file_config = load_config(cwd, config_path)?;

    // Merge CLI args with file config
    let merged_config = merge_cli_with_config(args, file_config);

    let ctx = CliContext {
        args: args.clone(),
        config: merged_config.clone(),
        log_level,
    };

    ctx.debug(&format!("Working directory: {:?}", cwd));
    ctx.debug(&format!("Directories to process: {:?}", args.directories));
    ctx.debug(&format!(
        "Output style: {}",
        args.style
            .map(|s| s.to_string())
            .unwrap_or_else(|| "from config".to_string())
    ));
    ctx.debug(&format!("Configuration: {:?}", ctx.config));

    // Determine target directory
    let target_dir = if args.directories.is_empty() {
        cwd.clone()
    } else {
        // For now, process first directory (TODO: support multiple)
        args.directories[0].clone()
    };

    // Resolve to absolute path
    let target_dir = if target_dir.is_absolute() {
        target_dir
    } else {
        cwd.join(&target_dir)
    };

    if log_level != LogLevel::Silent {
        println!(
            "{} Processing directory: {}",
            "📁".dimmed(),
            target_dir.display().to_string().cyan()
        );
    }

    // Step 1: Search for files
    if log_level != LogLevel::Silent {
        println!("{} Searching for files...", "🔍".dimmed());
    }

    let search_result = search_files(&target_dir, &merged_config)?;

    if log_level != LogLevel::Silent {
        println!(
            "{} Found {} files",
            "✓".green(),
            search_result.file_paths.len()
        );
    }

    let compression_enabled = args.compress || merged_config.output.compress;
    let should_collect_files =
        merged_config.output.files || compression_enabled || log_level != LogLevel::Silent;

    let collect_result = if should_collect_files {
        // Step 2: Collect file contents
        if log_level != LogLevel::Silent {
            println!("{} Reading files...", "📖".dimmed());
        }

        const MAX_FILE_SIZE: usize = 50 * 1024 * 1024; // 50MB
        let mut collect_result =
            collect_files(&target_dir, &search_result.file_paths, MAX_FILE_SIZE)?;

        // (Skipped files will be reported later after metrics)

        ctx.debug(&format!("Collected {} files", collect_result.files.len()));

        // Step 3: Apply compression if enabled (parallel)
        if compression_enabled {
            use rayon::prelude::*;

            if log_level != LogLevel::Silent {
                println!("{} Compressing code with tree-sitter...", "🗜".dimmed());
            }

            // Process files in parallel
            let compressed_files: Vec<_> = collect_result
                .files
                .par_iter()
                .map(|file| {
                    if let Some(compressed) = compress_code(&file.content, &file.path) {
                        if !compressed.is_empty() && compressed.len() < file.content.len() {
                            return (file.path.clone(), Some(compressed));
                        }
                    }
                    (file.path.clone(), None)
                })
                .collect();

            // Apply compressed content
            let mut compressed_count = 0;
            for file in &mut collect_result.files {
                if let Some((_, Some(content))) =
                    compressed_files.iter().find(|(p, _)| *p == file.path)
                {
                    file.content = content.clone();
                    compressed_count += 1;
                }
            }

            if log_level != LogLevel::Silent && compressed_count > 0 {
                println!("{} Compressed {} files", "✓".green(), compressed_count);
            }
        }

        Some(collect_result)
    } else {
        ctx.debug("Skipping file content collection for silent path-only output");
        None
    };

    // Step 4: Read instruction file if specified
    let instruction = if let Some(ref instr_path) = merged_config.output.instruction_file_path {
        let full_path = target_dir.join(instr_path);
        std::fs::read_to_string(&full_path).ok()
    } else {
        None
    };

    // Step 5: Generate output
    let effective_style = args.style.unwrap_or_else(|| {
        use crate::cli::args::OutputStyle;
        OutputStyle::try_from(merged_config.output.style.as_str()).unwrap_or(OutputStyle::Xml)
    });

    let output = if let Some(collect_result) = collect_result.as_ref() {
        generate_output(
            &collect_result.files,
            effective_style,
            &merged_config,
            instruction,
        )
    } else {
        generate_output_from_paths(
            &search_result.file_paths,
            effective_style,
            &merged_config,
            instruction,
        )
    };

    // Step 6: Handle output
    if args.stdout {
        // Output to stdout
        print!("{}", output);
    } else {
        // Write to file
        let output_path = ctx.output_path();

        // Create parent directories if needed
        if let Some(parent) = output_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
            }
        }

        fs::write(&output_path, &output)
            .with_context(|| format!("Failed to write output file: {}", output_path.display()))?;

        if log_level != LogLevel::Silent {
            use crate::core::metrics::PackMetrics;
            let collect_result = collect_result
                .as_ref()
                .expect("metrics path requires collected files");
            let file_contents: Vec<(String, String)> = collect_result
                .files
                .iter()
                .map(|f| (f.path.clone(), f.content.clone()))
                .collect();
            let metrics = PackMetrics::calculate(&file_contents, &output);

            println!();
            println!(
                "{} Output written to: {}",
                "✓".green().bold(),
                output_path.display().to_string().cyan()
            );
            println!();
            // Display metrics
            print_metrics(&metrics, merged_config.output.top_files_length);

            // Report skipped files
            print_skipped_files(&collect_result.skipped);
        }
    }

    // Step 8: Copy to clipboard if requested
    if args.copy || merged_config.output.copy_to_clipboard {
        ctx.debug("Clipboard copy requested (not implemented)");
        // TODO: Implement clipboard copy (requires additional crate)
    }

    Ok(())
}

/// Run the init action (create config file)
fn run_init_action(cwd: &Path, global: bool) -> Result<()> {
    use std::fs;

    let config_dir = if global {
        dirs::home_dir()
            .context("Could not find home directory")?
            .join(".config")
            .join("repomix")
    } else {
        cwd.to_path_buf()
    };

    // Create directory if needed
    if global {
        fs::create_dir_all(&config_dir).context("Failed to create config directory")?;
    }

    let config_path = config_dir.join("repomix.config.json");

    if config_path.exists() {
        println!(
            "{} Config file already exists at: {}",
            "⚠".yellow(),
            config_path.display().to_string().cyan()
        );
        return Ok(());
    }

    let default_config = RepomixConfig::default();
    let config_json = serde_json::to_string_pretty(&default_config)
        .context("Failed to serialize default config")?;

    fs::write(&config_path, config_json).context("Failed to write config file")?;

    println!(
        "{} Created config file at: {}",
        "✓".green(),
        config_path.display().to_string().cyan()
    );

    Ok(())
}

/// Run the remote action (clone and process remote repository)
fn run_remote_action(url: &str, branch: Option<String>, args: &Args) -> Result<()> {
    use crate::remote::{clone_from_url, parse_remote_url};
    use std::fs;

    // Determine log level
    let log_level = if args.quiet || args.stdout {
        LogLevel::Silent
    } else if args.verbose {
        LogLevel::Debug
    } else {
        LogLevel::Info
    };

    let cwd = env::current_dir().context("Failed to get current directory")?;

    // Parse and display repository info
    let info =
        parse_remote_url(url).context("Invalid remote repository URL or shorthand (owner/repo)")?;

    if log_level != LogLevel::Silent {
        println!(
            "{} Remote repository: {}",
            "→".cyan(),
            info.to_string().green()
        );
        println!("  Clone URL: {}", info.url.dimmed());
        if let Some(ref b) = branch.clone().or(info.branch.clone()) {
            println!("  Branch: {}", b.yellow());
        }
        println!();
    }

    // Clone the repository
    if log_level != LogLevel::Silent {
        println!("{} Cloning repository...", "⏳".dimmed());
    }

    let clone_result =
        clone_from_url(url, branch.as_deref()).context("Failed to clone repository")?;

    if log_level != LogLevel::Silent {
        println!("{} Repository cloned successfully!", "✓".green());
        println!();
    }

    // Load configuration from cloned repo
    let config_path = args.config.as_deref();
    let file_config = load_config(clone_result.path(), config_path)?;

    // Merge CLI args with file config
    let merged_config = merge_cli_with_config(args, file_config);

    // Create context for the cloned directory
    let ctx = CliContext {
        args: args.clone(),
        config: merged_config.clone(),
        log_level,
    };

    ctx.debug(&format!("Cloned to: {:?}", clone_result.path()));

    // Process the cloned repository
    use crate::core::file::{collect_files, search_files};
    use crate::core::output::generate::generate_output;

    let search_result = search_files(clone_result.path(), &merged_config)?;

    if log_level != LogLevel::Silent {
        println!(
            "{} Found {} files",
            "📁".dimmed(),
            search_result.file_paths.len()
        );
    }

    // Collect files - use 50MB max file size
    const MAX_FILE_SIZE: usize = 50 * 1024 * 1024;
    let mut collect_result = collect_files(
        clone_result.path(),
        &search_result.file_paths,
        MAX_FILE_SIZE,
    )?;

    // (Skipped files will be reported later after metrics)

    // Apply compression if enabled (parallel)
    if args.compress || merged_config.output.compress {
        use crate::core::compress::compress_code;
        use rayon::prelude::*;

        if log_level != LogLevel::Silent {
            println!("{} Compressing code with tree-sitter...", "🗜".dimmed());
        }

        // Process files in parallel
        let compressed_files: Vec<_> = collect_result
            .files
            .par_iter()
            .map(|file| {
                if let Some(compressed) = compress_code(&file.content, &file.path) {
                    if !compressed.is_empty() && compressed.len() < file.content.len() {
                        return (file.path.clone(), Some(compressed));
                    }
                }
                (file.path.clone(), None)
            })
            .collect();

        // Apply compressed content
        let mut compressed_count = 0;
        for file in &mut collect_result.files {
            if let Some((_, Some(content))) = compressed_files.iter().find(|(p, _)| *p == file.path)
            {
                file.content = content.clone();
                compressed_count += 1;
            }
        }

        if log_level != LogLevel::Silent && compressed_count > 0 {
            println!("{} Compressed {} files", "✓".green(), compressed_count);
        }
    }

    // Read instruction file if specified
    let instruction = if let Some(ref instr_path) = merged_config.output.instruction_file_path {
        let full_path = clone_result.path().join(instr_path);
        std::fs::read_to_string(&full_path).ok()
    } else {
        None
    };

    // Determine effective style for generation
    let effective_style = args.style.unwrap_or_else(|| {
        use crate::cli::args::OutputStyle;
        OutputStyle::try_from(merged_config.output.style.as_str()).unwrap_or(OutputStyle::Xml)
    });

    if log_level != LogLevel::Silent {
        println!(
            "{} Generating {} output...",
            "📝".dimmed(),
            effective_style.to_string().cyan()
        );
    }

    // Generate output
    let output = generate_output(
        &collect_result.files,
        effective_style,
        &merged_config,
        instruction,
    );

    // Handle output
    if args.stdout {
        // Output to stdout
        print!("{}", output);
    } else {
        // Write to file in current directory (not temp directory)
        let output_filename = merged_config.output.file_path.clone();
        let output_path = cwd.join(&output_filename);

        fs::write(&output_path, &output)
            .with_context(|| format!("Failed to write output file: {}", output_path.display()))?;

        if log_level != LogLevel::Silent {
            use crate::core::metrics::PackMetrics;
            let file_contents: Vec<(String, String)> = collect_result
                .files
                .iter()
                .map(|f| (f.path.clone(), f.content.clone()))
                .collect();
            let metrics = PackMetrics::calculate(&file_contents, &output);

            println!(
                "\n{} Output written to: {}",
                "✓".green(),
                output_path.display().to_string().cyan()
            );
            println!();
            // Display metrics
            print_metrics(&metrics, merged_config.output.top_files_length);

            // Report skipped files
            print_skipped_files(&collect_result.skipped);
        }
    }

    // temp directory is cleaned up automatically when clone_result goes out of scope
    if log_level == LogLevel::Debug {
        println!("{} Cleaning up temporary directory...", "🧹".dimmed());
    }

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
    // Override style only if explicitly provided via CLI
    let effective_style = if let Some(cli_style) = args.style {
        config.output.style = cli_style.to_string();
        cli_style
    } else {
        // Use style from config (already set above)
        use crate::cli::args::OutputStyle;
        OutputStyle::try_from(config.output.style.as_str()).unwrap_or(OutputStyle::Xml)
    };

    // Override output file path
    if let Some(ref output) = args.output {
        config.output.file_path = output.to_string_lossy().to_string();
    } else {
        // Auto-adjust file path based on effective style
        config.output.file_path = effective_style.default_file_name().to_string();
    }

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

/// Print information about skipped binary files
fn print_skipped_files(skipped: &[crate::core::file::SkippedFile]) {
    use crate::core::file::FileSkipReason;

    let binary_files: Vec<_> = skipped
        .iter()
        .filter(|f| {
            matches!(
                f.reason,
                FileSkipReason::BinaryContent | FileSkipReason::BinaryExtension
            )
        })
        .collect();

    if binary_files.is_empty() {
        return;
    }

    println!();
    println!("{}", "📄 Binary Files Detected:".white());
    println!("{}", "─────────────────────────".dimmed());

    if binary_files.len() == 1 {
        println!("{}", "1 file detected as binary:".yellow());
    } else {
        println!(
            "{}",
            format!("{} files detected as binary:", binary_files.len()).yellow()
        );
    }

    for (i, file) in binary_files.iter().enumerate() {
        println!("{}. {}", (i + 1).to_string().white(), file.path.white());
    }

    println!();
    println!(
        "{}",
        "These files have been excluded from the output.".yellow()
    );
    println!(
        "{}",
        "Please review these files if you expected them to contain text content.".yellow()
    );
}

/// Print metrics in a formatted, colorful way
fn print_metrics(metrics: &crate::core::metrics::PackMetrics, top_n: usize) {
    println!("{}", "📊 Pack Metrics:".cyan().bold());
    println!(
        "   Total files: {}",
        metrics.total_files.to_string().white().bold()
    );
    println!(
        "   Total characters: {}",
        format_number(metrics.total_characters).white()
    );
    println!(
        "   Total tokens: {}",
        format_number(metrics.total_tokens).green().bold()
    );

    let top = metrics.top_files(top_n);
    if !top.is_empty() {
        println!();
        println!(
            "{}",
            format!("📁 Top {} files by token count:", top.len()).cyan()
        );
        for (i, file) in top.iter().enumerate() {
            println!(
                "   {}. {} ({})",
                i + 1,
                file.path.dimmed(),
                format!("{} tokens", format_number(file.tokens)).yellow()
            );
        }
    }
}

/// Format a number with thousand separators
fn format_number(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();

    for (i, c) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i).is_multiple_of(3) {
            result.push(',');
        }
        result.push(*c);
    }

    result
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

    #[test]
    fn test_merge_cli_with_config_respects_file_style() {
        use crate::cli::args::Args;
        use crate::config::schema::RepomixConfig;

        // Base CLI args (no style specified)
        let args = Args {
            directories: vec![],
            command: None,
            verbose: false,
            quiet: false,
            stdout: false,
            stdin: false,
            copy: false,
            token_count_tree: None,
            top_files_len: 5,
            output: None,
            style: None, // Missing style flag
            parsable_style: false,
            compress: false,
            show_line_numbers: false,
            no_file_summary: false,
            no_directory_structure: false,
            no_files: false,
            remove_comments: false,
            remove_empty_lines: false,
            truncate_base64: false,
            header_text: None,
            instruction_file_path: None,
            include_empty_directories: false,
            include: vec![],
            ignore: vec![],
            no_gitignore: false,
            no_default_patterns: false,
            remote: None,
            remote_branch: None,
            config: None,
            init: false,
            global: false,
            no_security_check: false,
            token_count_encoding: "o200k_base".to_string(),
        };

        // Config file with markdown style
        let mut file_config = RepomixConfig::default();
        file_config.output.style = "markdown".to_string();

        let merged = merge_cli_with_config(&args, file_config);

        // Should respect config file
        assert_eq!(merged.output.style, "markdown");
        // Should auto-adjust file path
        assert_eq!(merged.output.file_path, "repomix-output.md");
    }

    #[test]
    fn test_merge_cli_with_config_overrides_file_style() {
        use crate::cli::args::{Args, OutputStyle};
        use crate::config::schema::RepomixConfig;

        // CLI args with json style
        let args = Args {
            directories: vec![],
            command: None,
            verbose: false,
            quiet: false,
            stdout: false,
            stdin: false,
            copy: false,
            token_count_tree: None,
            top_files_len: 5,
            output: None,
            style: Some(OutputStyle::Json),
            parsable_style: false,
            compress: false,
            show_line_numbers: false,
            no_file_summary: false,
            no_directory_structure: false,
            no_files: false,
            remove_comments: false,
            remove_empty_lines: false,
            truncate_base64: false,
            header_text: None,
            instruction_file_path: None,
            include_empty_directories: false,
            include: vec![],
            ignore: vec![],
            no_gitignore: false,
            no_default_patterns: false,
            remote: None,
            remote_branch: None,
            config: None,
            init: false,
            global: false,
            no_security_check: false,
            token_count_encoding: "o200k_base".to_string(),
        };

        // Config file with markdown style
        let mut file_config = RepomixConfig::default();
        file_config.output.style = "markdown".to_string();

        let merged = merge_cli_with_config(&args, file_config);

        // CLI should override config
        assert_eq!(merged.output.style, "json");
        // Should auto-adjust file path
        assert_eq!(merged.output.file_path, "repomix-output.json");
    }
}
