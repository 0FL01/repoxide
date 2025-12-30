//! Configuration file loader
//!
//! This module handles loading repomix configuration from various file formats.

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use super::schema::RepomixConfig;

/// Supported config file names in priority order
const CONFIG_FILE_NAMES: &[&str] = &[
    "repomix.config.json",
    "repomix.config.jsonc",
    "repomix.config.json5",
];

/// Global config directory name
const GLOBAL_CONFIG_DIR: &str = "repomix";

/// Load configuration from a directory
///
/// This function searches for a config file in the following order:
/// 1. If a specific path is provided, use that
/// 2. Search for local config files in the directory
/// 3. Search for global config files in ~/.config/repomix/
/// 4. Return default configuration if no config found
pub fn load_config(directory: &Path, config_path: Option<&Path>) -> Result<RepomixConfig> {
    // If a specific config path is provided, use it
    if let Some(path) = config_path {
        let full_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            directory.join(path)
        };

        if full_path.exists() {
            return load_config_file(&full_path);
        }
        anyhow::bail!("Config file not found at: {:?}", full_path);
    }

    // Search for local config files
    for config_name in CONFIG_FILE_NAMES {
        let config_path = directory.join(config_name);
        if config_path.exists() {
            return load_config_file(&config_path);
        }
    }

    // Search for global config files
    if let Some(global_config) = find_global_config() {
        return load_config_file(&global_config);
    }

    // Return default config if no config file found
    Ok(RepomixConfig::default())
}

/// Find global config file
fn find_global_config() -> Option<std::path::PathBuf> {
    let config_dir = dirs::config_dir()?.join(GLOBAL_CONFIG_DIR);

    for config_name in CONFIG_FILE_NAMES {
        let config_path = config_dir.join(config_name);
        if config_path.exists() {
            return Some(config_path);
        }
    }

    // Also check home directory ~/.repomix/
    let home_config_dir = dirs::home_dir()?.join(format!(".{}", GLOBAL_CONFIG_DIR));
    for config_name in CONFIG_FILE_NAMES {
        let config_path = home_config_dir.join(config_name);
        if config_path.exists() {
            return Some(config_path);
        }
    }

    None
}

/// Load configuration from a specific file
fn load_config_file(path: &Path) -> Result<RepomixConfig> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {:?}", path))?;

    let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    match extension {
        "json" => parse_json(&content, path),
        "jsonc" | "json5" => parse_jsonc(&content, path),
        _ => parse_json(&content, path), // Default to JSON
    }
}

/// Parse standard JSON config
fn parse_json(content: &str, path: &Path) -> Result<RepomixConfig> {
    serde_json::from_str(content)
        .with_context(|| format!("Failed to parse JSON config file: {:?}", path))
}

/// Parse JSONC (JSON with comments) or JSON5 config
fn parse_jsonc(content: &str, path: &Path) -> Result<RepomixConfig> {
    // Strip single-line comments (// ...)
    let without_line_comments: String = content
        .lines()
        .map(|line| {
            if let Some(idx) = line.find("//") {
                // Check if it's inside a string
                let before_comment = &line[..idx];
                let quote_count = before_comment.matches('"').count();
                if quote_count % 2 == 0 {
                    // Not inside a string, remove comment
                    before_comment
                } else {
                    line
                }
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    // Strip block comments (/* ... */)
    let without_block_comments = strip_block_comments(&without_line_comments);

    // Handle trailing commas (JSON5 feature)
    let cleaned = remove_trailing_commas(&without_block_comments);

    serde_json::from_str(&cleaned)
        .with_context(|| format!("Failed to parse JSONC/JSON5 config file: {:?}", path))
}

/// Remove block comments from content
fn strip_block_comments(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let mut chars = content.chars().peekable();
    let mut in_string = false;
    let mut prev_char = None;

    while let Some(c) = chars.next() {
        if in_string {
            result.push(c);
            if c == '"' && prev_char != Some('\\') {
                in_string = false;
            }
        } else if c == '"' {
            result.push(c);
            in_string = true;
        } else if c == '/' && chars.peek() == Some(&'*') {
            // Start of block comment
            chars.next(); // consume '*'
                          // Skip until we find */
            let mut depth = 1;
            while depth > 0 {
                if let Some(next) = chars.next() {
                    if next == '*' && chars.peek() == Some(&'/') {
                        chars.next();
                        depth -= 1;
                    } else if next == '/' && chars.peek() == Some(&'*') {
                        chars.next();
                        depth += 1;
                    }
                } else {
                    break;
                }
            }
        } else {
            result.push(c);
        }
        prev_char = Some(c);
    }

    result
}

/// Remove trailing commas before } or ]
fn remove_trailing_commas(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let chars: Vec<char> = content.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let c = chars[i];

        if c == ',' {
            // Look ahead for } or ]
            let mut j = i + 1;
            while j < len && chars[j].is_whitespace() {
                j += 1;
            }

            if j < len && (chars[j] == '}' || chars[j] == ']') {
                // Skip this comma
                i += 1;
                continue;
            }
        }

        result.push(c);
        i += 1;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_load_default_config() {
        let dir = tempdir().unwrap();
        let config = load_config(dir.path(), None).unwrap();
        assert_eq!(config.output.file_path, "repomix-output.xml");
    }

    #[test]
    fn test_load_json_config() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("repomix.config.json");

        let mut file = fs::File::create(&config_path).unwrap();
        writeln!(file, r#"{{"output": {{"filePath": "custom.xml"}}}}"#).unwrap();

        let config = load_config(dir.path(), None).unwrap();
        assert_eq!(config.output.file_path, "custom.xml");
    }

    #[test]
    fn test_load_jsonc_config() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("repomix.config.jsonc");

        let mut file = fs::File::create(&config_path).unwrap();
        writeln!(
            file,
            r#"{{
            // This is a comment
            "output": {{
                "filePath": "commented.xml" // inline comment
            }}
        }}"#
        )
        .unwrap();

        let config = load_config(dir.path(), None).unwrap();
        assert_eq!(config.output.file_path, "commented.xml");
    }

    #[test]
    fn test_load_config_with_trailing_commas() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("repomix.config.json5");

        let mut file = fs::File::create(&config_path).unwrap();
        writeln!(
            file,
            r#"{{
            "output": {{
                "filePath": "trailing.xml",
            }},
            "include": [
                "src/**",
            ],
        }}"#
        )
        .unwrap();

        let config = load_config(dir.path(), None).unwrap();
        assert_eq!(config.output.file_path, "trailing.xml");
    }

    #[test]
    fn test_strip_block_comments() {
        let input = r#"{ /* comment */ "key": "value" }"#;
        let result = strip_block_comments(input);
        assert_eq!(result, r#"{  "key": "value" }"#);
    }

    #[test]
    fn test_remove_trailing_commas() {
        let input = r#"{"a": 1, "b": 2,}"#;
        let result = remove_trailing_commas(input);
        assert_eq!(result, r#"{"a": 1, "b": 2}"#);
    }

    #[test]
    fn test_specific_config_path() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("my-config.json");

        let mut file = fs::File::create(&config_path).unwrap();
        writeln!(file, r#"{{"output": {{"filePath": "specific.xml"}}}}"#).unwrap();

        let config = load_config(dir.path(), Some(Path::new("my-config.json"))).unwrap();
        assert_eq!(config.output.file_path, "specific.xml");
    }

    #[test]
    fn test_missing_specific_config() {
        let dir = tempdir().unwrap();
        let result = load_config(dir.path(), Some(Path::new("nonexistent.json")));
        assert!(result.is_err());
    }
}
