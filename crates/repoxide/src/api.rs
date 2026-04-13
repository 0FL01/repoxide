//! Public API for programmatic usage (web server, integrations)
//!
//! This module provides a clean API for using the Rust repoxide crate as a library,
//! separate from the CLI interface.

use anyhow::{Context, Result};
use std::path::Path;
use std::time::Instant;

pub use crate::cli::args::OutputStyle;
pub use crate::config::MergedConfig;
pub use crate::core::metrics::{PackMetrics, PackPhaseTimings};

/// Result of packing operation
#[derive(Debug, Clone)]
pub struct PackResult {
    /// Generated output content (XML/Markdown/Plain/JSON)
    pub content: String,
    /// Metrics: files, tokens, characters
    pub metrics: PackMetrics,
    /// Output format used
    pub format: OutputStyle,
    /// List of processed file paths
    pub file_paths: Vec<String>,
    /// Per-phase wall-clock timings for this pack run
    pub phase_timings: PackPhaseTimings,
}

fn elapsed_ms(start: Instant) -> u64 {
    start.elapsed().as_millis() as u64
}

fn pack_prepared_directory(root_path: &Path, config: MergedConfig) -> Result<PackResult> {
    use crate::core::compress::compress_files_in_place;
    use crate::core::file::{collect_files, search_files};
    use crate::core::output::generate::generate_output;

    let total_start = Instant::now();
    let mut phase_timings = PackPhaseTimings::default();

    let search_started = Instant::now();
    let search_result = search_files(root_path, &config).context("Failed to search files")?;
    phase_timings.search_ms = elapsed_ms(search_started);

    const MAX_FILE_SIZE: usize = 50 * 1024 * 1024; // 50MB
    let collect_started = Instant::now();
    let mut collect_result = collect_files(root_path, &search_result.file_paths, MAX_FILE_SIZE)
        .context("Failed to collect files")?;
    phase_timings.collect_ms = elapsed_ms(collect_started);

    if config.output.compress {
        let compress_started = Instant::now();
        compress_files_in_place(&mut collect_result.files);
        phase_timings.compress_ms = elapsed_ms(compress_started);
    }

    let instruction = if let Some(ref instr_path) = config.output.instruction_file_path {
        let full_path = root_path.join(instr_path);
        std::fs::read_to_string(&full_path).ok()
    } else {
        None
    };

    let effective_style =
        OutputStyle::try_from(config.output.style.as_str()).unwrap_or(OutputStyle::Xml);

    let output_started = Instant::now();
    let output = generate_output(&collect_result.files, effective_style, &config, instruction);
    phase_timings.output_ms = elapsed_ms(output_started);

    let metrics_started = Instant::now();
    let metrics = PackMetrics::calculate(&collect_result.files, &output);
    phase_timings.metrics_ms = elapsed_ms(metrics_started);
    phase_timings.total_ms = elapsed_ms(total_start);

    let file_paths = collect_result
        .files
        .into_iter()
        .map(|file| file.path)
        .collect();

    Ok(PackResult {
        content: output,
        metrics,
        format: effective_style,
        file_paths,
        phase_timings,
    })
}

/// Options for packing (mirror of web API options)
#[derive(Debug, Clone)]
pub struct PackOptions {
    pub format: Option<OutputStyle>,
    pub compress: bool,
    pub remove_comments: bool,
    pub remove_empty_lines: bool,
    pub show_line_numbers: bool,
    pub file_summary: bool,
    pub directory_structure: bool,
    pub include_patterns: Option<String>,
    pub ignore_patterns: Option<String>,
    pub output_parsable: bool,
    pub header_text: Option<String>,
    pub instruction_file_path: Option<String>,
}

impl Default for PackOptions {
    fn default() -> Self {
        Self {
            format: None,
            compress: false,
            remove_comments: false,
            remove_empty_lines: false,
            show_line_numbers: false,
            file_summary: true,
            directory_structure: true,
            include_patterns: None,
            ignore_patterns: None,
            output_parsable: false,
            header_text: None,
            instruction_file_path: None,
        }
    }
}

/// Pack a local directory
///
/// # Arguments
/// * `root_path` - Path to the directory to pack
/// * `config` - Merged configuration (use `build_config` to create from `PackOptions`)
///
/// # Returns
/// `PackResult` with generated output and metrics
pub fn pack_directory(root_path: &Path, config: MergedConfig) -> Result<PackResult> {
    pack_prepared_directory(root_path, config)
}

/// Pack a remote repository
///
/// # Arguments
/// * `url` - Repository URL or shorthand (e.g., "user/repo" or full GitHub URL)
/// * `branch` - Optional branch name
/// * `config` - Merged configuration
///
/// # Returns
/// `PackResult` with generated output and metrics
pub fn pack_remote(url: &str, branch: Option<&str>, config: MergedConfig) -> Result<PackResult> {
    use crate::remote::{clone_from_url, parse_remote_url};

    // Parse repository URL
    let _info = parse_remote_url(url).context("Invalid remote repository URL or shorthand")?;

    // Clone the repository
    let clone_result = clone_from_url(url, branch).context("Failed to clone repository")?;

    pack_prepared_directory(clone_result.path(), config)
}

/// Build MergedConfig from PackOptions
///
/// # Arguments
/// * `options` - Pack options from web API or other programmatic usage
///
/// # Returns
/// `MergedConfig` ready for use with `pack_directory` or `pack_remote`
pub fn build_config(options: PackOptions) -> MergedConfig {
    use crate::config::schema::*;

    let mut config = MergedConfig::default();

    // Apply format
    if let Some(format) = options.format {
        config.output.style = format.to_string();
        config.output.file_path = format.default_file_name().to_string();
    }

    // Apply boolean flags
    config.output.compress = options.compress;
    config.output.remove_comments = options.remove_comments;
    config.output.remove_empty_lines = options.remove_empty_lines;
    config.output.show_line_numbers = options.show_line_numbers;
    config.output.file_summary = options.file_summary;
    config.output.directory_structure = options.directory_structure;
    config.output.parsable_style = options.output_parsable;

    // Apply header text
    if let Some(header) = options.header_text {
        config.output.header_text = Some(header);
    }

    // Apply instruction file path
    if let Some(instr_path) = options.instruction_file_path {
        config.output.instruction_file_path = Some(instr_path);
    }

    // Parse and apply include patterns
    if let Some(patterns) = options.include_patterns {
        config.include = split_pattern_list(&patterns);
    }

    // Parse and apply ignore patterns
    if let Some(patterns) = options.ignore_patterns {
        config.ignore.custom_patterns = split_pattern_list(&patterns);
    }

    config
}

fn split_pattern_list(patterns: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut escaped = false;

    for ch in patterns.chars() {
        if escaped {
            if ch == ',' {
                current.push(',');
            } else {
                current.push('\\');
                current.push(ch);
            }
            escaped = false;
            continue;
        }

        if ch == '\\' {
            escaped = true;
            continue;
        }

        if ch == ',' {
            let trimmed = current.trim();
            if !trimmed.is_empty() {
                result.push(trimmed.to_string());
            }
            current.clear();
            continue;
        }

        current.push(ch);
    }

    if escaped {
        current.push('\\');
    }

    let trimmed = current.trim();
    if !trimmed.is_empty() {
        result.push(trimmed.to_string());
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_config_defaults() {
        let options = PackOptions::default();
        let config = build_config(options);

        assert_eq!(config.output.style, "xml");
        assert!(!config.output.compress);
        assert!(!config.output.remove_comments);
        assert!(config.output.file_summary);
        assert!(config.output.directory_structure);
        assert!(!config.output.parsable_style);
    }

    #[test]
    fn test_build_config_with_options() {
        let options = PackOptions {
            format: Some(OutputStyle::Markdown),
            compress: true,
            remove_comments: true,
            remove_empty_lines: true,
            show_line_numbers: true,
            file_summary: false,
            directory_structure: false,
            include_patterns: Some("*.rs,*.toml".to_string()),
            ignore_patterns: Some("target/**,*.log".to_string()),
            output_parsable: true,
            header_text: Some("Custom header".to_string()),
            instruction_file_path: Some("INSTRUCTIONS.md".to_string()),
        };

        let config = build_config(options);

        assert_eq!(config.output.style, "markdown");
        assert_eq!(config.output.file_path, "repoxide-output.md");
        assert!(config.output.compress);
        assert!(config.output.remove_comments);
        assert!(config.output.remove_empty_lines);
        assert!(config.output.show_line_numbers);
        assert!(!config.output.file_summary);
        assert!(!config.output.directory_structure);
        assert!(config.output.parsable_style);
        assert_eq!(config.include, vec!["*.rs", "*.toml"]);
        assert_eq!(config.ignore.custom_patterns, vec!["target/**", "*.log"]);
        assert_eq!(config.output.header_text, Some("Custom header".to_string()));
        assert_eq!(
            config.output.instruction_file_path,
            Some("INSTRUCTIONS.md".to_string())
        );
    }

    #[test]
    fn test_pack_options_default() {
        let options = PackOptions::default();
        assert_eq!(options.format, None);
        assert!(!options.compress);
    }

    #[test]
    fn test_split_pattern_list_keeps_escaped_commas() {
        let patterns =
            split_pattern_list(r"src/[[]literal].rs,docs/file\,with\,comma.md,**/*.toml");

        assert_eq!(
            patterns,
            vec![
                "src/[[]literal].rs".to_string(),
                "docs/file,with,comma.md".to_string(),
                "**/*.toml".to_string(),
            ]
        );
    }
}
