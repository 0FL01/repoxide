//! File search functionality
//!
//! Provides directory walking with glob pattern matching and gitignore support.

use anyhow::{Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

use crate::config::MergedConfig;

/// Default ignore patterns (mirrors defaultIgnore.ts)
pub const DEFAULT_IGNORE_PATTERNS: &[&str] = &[
    // Version control
    ".git/**",
    ".hg/**",
    ".hgignore",
    ".svn/**",
    // Dependency directories
    "**/node_modules/**",
    "**/bower_components/**",
    "**/jspm_packages/**",
    "vendor/**",
    "**/.bundle/**",
    "**/.gradle/**",
    "target/**",
    // Logs
    "logs/**",
    "**/*.log",
    "**/npm-debug.log*",
    "**/yarn-debug.log*",
    "**/yarn-error.log*",
    // Runtime data
    "pids/**",
    "*.pid",
    "*.seed",
    "*.pid.lock",
    // Coverage
    "lib-cov/**",
    "coverage/**",
    ".nyc_output/**",
    // Build tools
    ".grunt/**",
    ".lock-wscript",
    "build/Release/**",
    "typings/**",
    // Cache directories
    "**/.npm/**",
    ".eslintcache",
    ".rollup.cache/**",
    ".webpack.cache/**",
    ".parcel-cache/**",
    ".sass-cache/**",
    "*.cache",
    // Node
    ".node_repl_history",
    "*.tgz",
    "**/.yarn/**",
    "**/.yarn-integrity",
    // Environment
    ".env",
    // Framework outputs
    ".next/**",
    ".nuxt/**",
    ".vuepress/dist/**",
    ".serverless/**",
    ".fusebox/**",
    ".dynamodb/**",
    "**/dist/**",
    // OS generated
    "**/.DS_Store",
    "**/Thumbs.db",
    // Editor files
    ".idea/**",
    ".vscode/**",
    "**/*.swp",
    "**/*.swo",
    "**/*.swn",
    "**/*.bak",
    // Build outputs
    "**/build/**",
    "**/out/**",
    // Temp files
    "tmp/**",
    "temp/**",
    // Repomix output
    "**/repomix-output.*",
    "**/repopack-output.*",
    // Lock files
    "**/package-lock.json",
    "**/yarn-error.log",
    "**/yarn.lock",
    "**/pnpm-lock.yaml",
    "**/bun.lockb",
    "**/bun.lock",
    // Python
    "**/__pycache__/**",
    "**/*.py[cod]",
    "**/venv/**",
    "**/.venv/**",
    "**/.pytest_cache/**",
    "**/.mypy_cache/**",
    "**/.ipynb_checkpoints/**",
    "**/Pipfile.lock",
    "**/poetry.lock",
    "**/uv.lock",
    // Rust
    "**/Cargo.lock",
    "**/Cargo.toml.orig",
    "**/target/**",
    "**/*.rs.bk",
    // PHP
    "**/composer.lock",
    // Ruby
    "**/Gemfile.lock",
    // Go
    "**/go.sum",
    // Elixir
    "**/mix.lock",
    // Haskell
    "**/stack.yaml.lock",
    "**/cabal.project.freeze",
];

/// Search result containing found files and empty directories
#[derive(Debug, Clone)]
pub struct FileSearchResult {
    /// List of file paths relative to root directory
    pub file_paths: Vec<String>,
    /// List of empty directory paths relative to root directory
    pub empty_dir_paths: Vec<String>,
}

/// Build a GlobSet from patterns
fn build_glob_set(patterns: &[String]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let glob = Glob::new(pattern)
            .with_context(|| format!("Invalid glob pattern: {}", pattern))?;
        builder.add(glob);
    }
    builder.build().context("Failed to build glob set")
}

/// Normalize a glob pattern for consistent matching
fn normalize_pattern(pattern: &str) -> String {
    let mut p = pattern.to_string();
    
    // Remove trailing slash (except for **/)
    if p.ends_with('/') && !p.ends_with("**/") {
        p = p[..p.len() - 1].to_string();
    }
    
    // Convert **/folder to **/folder/** for consistent directory matching
    if p.starts_with("**/") && !p.contains("/**") && !p.contains('*') {
        p = format!("{}/**", p);
    }
    
    p
}

/// Check if a path matches any pattern in the glob set
fn matches_any(path: &str, glob_set: &GlobSet) -> bool {
    glob_set.is_match(path)
}

/// Search for files in a directory with filtering based on configuration
pub fn search_files(root_dir: &Path, config: &MergedConfig) -> Result<FileSearchResult> {
    // Validate root directory exists
    if !root_dir.exists() {
        anyhow::bail!("Target path does not exist: {}", root_dir.display());
    }
    
    if !root_dir.is_dir() {
        anyhow::bail!(
            "Target path is not a directory: {}. Please specify a directory path.",
            root_dir.display()
        );
    }

    // Build ignore patterns
    let mut ignore_patterns: Vec<String> = Vec::new();
    
    // Add default ignore patterns if enabled
    if config.ignore.use_default_patterns {
        for pattern in DEFAULT_IGNORE_PATTERNS {
            ignore_patterns.push(pattern.to_string());
        }
    }
    
    // Add output file to ignore patterns
    if !config.output.file_path.is_empty() {
        ignore_patterns.push(config.output.file_path.clone());
    }
    
    // Add custom ignore patterns
    for pattern in &config.ignore.custom_patterns {
        ignore_patterns.push(normalize_pattern(pattern));
    }
    
    // Normalize ignore patterns
    let ignore_patterns: Vec<String> = ignore_patterns
        .iter()
        .map(|p| normalize_pattern(p))
        .collect();
    
    // Build glob sets
    let ignore_glob_set = build_glob_set(&ignore_patterns)?;
    
    // Build include glob set
    let include_patterns: Vec<String> = if config.include.is_empty() {
        vec!["**/*".to_string()]
    } else {
        config.include.clone()
    };
    let include_glob_set = build_glob_set(&include_patterns)?;
    
    // Use ignore crate for walking with gitignore support
    let mut builder = WalkBuilder::new(root_dir);
    builder
        .hidden(false) // Include dotfiles
        .git_ignore(config.ignore.use_gitignore)
        .git_global(false)
        .git_exclude(config.ignore.use_gitignore)
        .follow_links(false)
        .parents(false);
    
    // Add .ignore file support if enabled
    if config.ignore.use_dot_ignore {
        builder.add_custom_ignore_filename(".ignore");
    }
    builder.add_custom_ignore_filename(".repomixignore");
    
    let mut file_paths: Vec<String> = Vec::new();
    let mut all_dirs: Vec<String> = Vec::new();
    let mut dirs_with_files: std::collections::HashSet<String> = std::collections::HashSet::new();
    
    for entry in builder.build() {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();
        
        // Get relative path
        let rel_path = match path.strip_prefix(root_dir) {
            Ok(p) => p,
            Err(_) => continue,
        };
        
        // Skip root directory
        if rel_path.as_os_str().is_empty() {
            continue;
        }
        
        let rel_path_str = rel_path.to_string_lossy().replace('\\', "/");
        
        // Check if ignored
        if matches_any(&rel_path_str, &ignore_glob_set) {
            continue;
        }
        
        if path.is_dir() {
            all_dirs.push(rel_path_str);
        } else if path.is_file() {
            // Check if matches include patterns
            if matches_any(&rel_path_str, &include_glob_set) {
                file_paths.push(rel_path_str.clone());
                
                // Mark parent directories as non-empty
                let mut parent = rel_path.parent();
                while let Some(p) = parent {
                    if p.as_os_str().is_empty() {
                        break;
                    }
                    dirs_with_files.insert(p.to_string_lossy().replace('\\', "/"));
                    parent = p.parent();
                }
            }
        }
    }
    
    // Find empty directories
    let empty_dir_paths: Vec<String> = if config.output.include_empty_directories {
        all_dirs
            .into_iter()
            .filter(|d| !dirs_with_files.contains(d))
            .collect()
    } else {
        Vec::new()
    };
    
    // Sort paths for consistent output
    let mut file_paths = file_paths;
    file_paths.sort();
    
    let mut empty_dir_paths = empty_dir_paths;
    empty_dir_paths.sort();
    
    Ok(FileSearchResult {
        file_paths,
        empty_dir_paths,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_default_ignore_patterns_not_empty() {
        assert!(!DEFAULT_IGNORE_PATTERNS.is_empty());
    }

    #[test]
    fn test_normalize_pattern() {
        assert_eq!(normalize_pattern("src/"), "src");
        assert_eq!(normalize_pattern("**/folder"), "**/folder");
    }

    #[test]
    fn test_build_glob_set() {
        let patterns = vec!["*.rs".to_string(), "**/*.txt".to_string()];
        let glob_set = build_glob_set(&patterns).unwrap();
        
        assert!(glob_set.is_match("main.rs"));
        assert!(glob_set.is_match("src/test.txt"));
        assert!(!glob_set.is_match("main.py"));
    }

    #[test]
    fn test_search_files() -> Result<()> {
        let dir = tempdir()?;
        let root = dir.path();
        
        // Create test files
        fs::write(root.join("main.rs"), "fn main() {}")?;
        fs::write(root.join("lib.rs"), "pub mod lib;")?;
        fs::create_dir(root.join("src"))?;
        fs::write(root.join("src/mod.rs"), "mod test;")?;
        
        let config = MergedConfig {
            cwd: root.to_string_lossy().to_string(),
            ..Default::default()
        };
        
        let result = search_files(root, &config)?;
        
        assert!(!result.file_paths.is_empty());
        assert!(result.file_paths.iter().any(|p| p.ends_with("main.rs")));
        assert!(result.file_paths.iter().any(|p| p.ends_with("lib.rs")));
        
        Ok(())
    }

    #[test]
    fn test_search_with_include_pattern() -> Result<()> {
        let dir = tempdir()?;
        let root = dir.path();
        
        fs::write(root.join("main.rs"), "fn main() {}")?;
        fs::write(root.join("readme.txt"), "README")?;
        
        let mut config = MergedConfig {
            cwd: root.to_string_lossy().to_string(),
            ..Default::default()
        };
        config.include = vec!["*.rs".to_string()];
        
        let result = search_files(root, &config)?;
        
        // Should only include .rs files
        assert!(result.file_paths.iter().all(|p| p.ends_with(".rs")));
        
        Ok(())
    }

    #[test]
    fn test_search_nonexistent_directory() {
        let config = MergedConfig::default();
        let result = search_files(Path::new("/nonexistent/path/123456"), &config);
        assert!(result.is_err());
    }
}
