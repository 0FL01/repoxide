//! Output generation orchestration
//!
//! Main entry point for generating output in various formats (XML, Markdown, JSON, Plain).

use crate::cli::args::OutputStyle;
use crate::config::MergedConfig;
use crate::core::file::collect::CollectedFile;
use crate::core::file::tree::generate_tree;

use super::json::generate_json;
use super::markdown::generate_markdown;
use super::plain::generate_plain;
use super::xml::generate_xml;

/// Processed file ready for output generation
#[derive(Debug, Clone)]
pub struct ProcessedFile {
    /// Relative path of the file
    pub path: String,
    /// Content of the file (possibly compressed/modified)
    pub content: String,
}

impl From<CollectedFile> for ProcessedFile {
    fn from(file: CollectedFile) -> Self {
        Self {
            path: file.path,
            content: file.content,
        }
    }
}

impl From<&CollectedFile> for ProcessedFile {
    fn from(file: &CollectedFile) -> Self {
        Self {
            path: file.path.clone(),
            content: file.content.clone(),
        }
    }
}

/// Context for output generation
#[derive(Debug, Clone)]
pub struct OutputContext {
    /// Generation date in ISO format

    /// Directory tree string
    pub tree_string: String,
    /// Processed files
    pub files: Vec<ProcessedFile>,
    /// File line counts (path -> count)

    /// Configuration
    pub config: OutputContextConfig,
    /// Optional instruction text
    pub instruction: Option<String>,
    /// Optional header text
    pub header_text: Option<String>,
}

/// Configuration options relevant to output generation
#[derive(Debug, Clone)]
pub struct OutputContextConfig {
    /// Include file summary section
    pub file_summary: bool,
    /// Include directory structure section
    pub directory_structure: bool,
    /// Include files section
    pub files: bool,
    /// Enable parsable style (escape special chars)
    pub parsable_style: bool,
    /// Show line numbers in output
    pub show_line_numbers: bool,
    /// Compression is enabled
    pub compress: bool,
    /// Comments were removed
    pub remove_comments: bool,
    /// Empty lines were removed
    pub remove_empty_lines: bool,
    /// Security check enabled
    pub security_check: bool,
    /// Include patterns
    pub include_patterns: Vec<String>,
    /// Ignore patterns
    pub ignore_patterns: Vec<String>,
    /// Use gitignore
    pub use_gitignore: bool,
    /// Use default patterns
    pub use_default_patterns: bool,
    /// Base64 truncated
    pub truncate_base64: bool,
}

impl Default for OutputContextConfig {
    fn default() -> Self {
        Self {
            file_summary: true,
            directory_structure: true,
            files: true,
            parsable_style: false,
            show_line_numbers: false,
            compress: false,
            remove_comments: false,
            remove_empty_lines: false,
            security_check: true,
            include_patterns: Vec::new(),
            ignore_patterns: Vec::new(),
            use_gitignore: true,
            use_default_patterns: true,
            truncate_base64: false,
        }
    }
}

/// Build output context from collected files and configuration
pub fn build_output_context(
    files: &[CollectedFile],
    config: &MergedConfig,
    instruction: Option<String>,
) -> OutputContext {


    // Generate directory tree
    let file_paths: Vec<String> = files.iter().map(|f| f.path.clone()).collect();
    let tree_string = generate_tree(&file_paths, &[]);

    // Convert to processed files
    let processed_files: Vec<ProcessedFile> = files.iter().map(ProcessedFile::from).collect();

    // Build context config
    let context_config = OutputContextConfig {
        file_summary: config.output.file_summary,
        directory_structure: config.output.directory_structure,
        files: config.output.files,
        parsable_style: config.output.parsable_style,
        show_line_numbers: config.output.show_line_numbers,
        compress: config.output.compress,
        remove_comments: config.output.remove_comments,
        remove_empty_lines: config.output.remove_empty_lines,
        security_check: config.security.enable_security_check,
        include_patterns: config.include.clone(),
        ignore_patterns: config.ignore.custom_patterns.clone(),
        use_gitignore: config.ignore.use_gitignore,
        use_default_patterns: config.ignore.use_default_patterns,
        truncate_base64: config.output.truncate_base64,
    };

    OutputContext {
        // generation_date: chrono::Utc::now().to_rfc3339(),
        tree_string,
        files: processed_files,
        // file_line_counts,
        config: context_config,
        instruction,
        header_text: config.output.header_text.clone(),
    }
}

/// Generate content header description
pub fn generate_header(config: &OutputContextConfig) -> String {
    // Generate selection description
    let is_entire_codebase = config.include_patterns.is_empty() && config.ignore_patterns.is_empty();
    
    let description = if is_entire_codebase {
        "This file is a merged representation of the entire codebase".to_string()
    } else {
        let mut parts = Vec::new();
        if !config.include_patterns.is_empty() {
            parts.push("specifically included files");
        }
        if !config.ignore_patterns.is_empty() {
            parts.push("files not matching ignore patterns");
        }
        format!(
            "This file is a merged representation of a subset of the codebase, containing {}",
            parts.join(" and ")
        )
    };

    // Add processing information
    let mut processing_notes = Vec::new();
    if config.remove_comments {
        processing_notes.push("comments have been removed");
    }
    if config.remove_empty_lines {
        processing_notes.push("empty lines have been removed");
    }
    if config.show_line_numbers {
        processing_notes.push("line numbers have been added");
    }
    if config.compress {
        processing_notes.push("content has been compressed (code blocks are separated by ⋮---- delimiter)");
    }
    if !config.security_check {
        processing_notes.push("security check has been disabled");
    }

    if processing_notes.is_empty() {
        format!("{}, combined into a single document by Repomix.", description)
    } else {
        format!(
            "{}, combined into a single document by Repomix.\nThe content has been processed where {}.",
            description,
            processing_notes.join(", ")
        )
    }
}

/// Generate summary purpose text
pub fn generate_summary_purpose(config: &OutputContextConfig) -> String {
    let content_description = if config.include_patterns.is_empty() && config.ignore_patterns.is_empty() {
        "the entire repository's contents"
    } else {
        "a subset of the repository's contents that is considered the most important context"
    };

    format!(
        "This file contains a packed representation of {}.\n\
         It is designed to be easily consumable by AI systems for analysis, code review,\n\
         or other automated processes.",
        content_description
    )
}

/// Generate file format description
pub fn generate_summary_file_format() -> String {
    "The content is organized as follows:\n\
     1. This summary section\n\
     2. Repository information\n\
     3. Directory structure\n\
     4. Repository files (if enabled)"
        .to_string()
}

/// Generate usage guidelines
pub fn generate_summary_usage_guidelines(has_header: bool, has_instruction: bool) -> String {
    let mut lines = vec![
        "- This file should be treated as read-only. Any changes should be made to the\n  original repository files, not this packed version.",
        "- When processing this file, use the file path to distinguish\n  between different files in the repository.",
        "- Be aware that this file may contain sensitive information. Handle it with\n  the same level of security as you would the original repository.",
    ];

    if has_header {
        lines.push("- Pay special attention to the Repository Description. These contain important context and guidelines specific to this project.");
    }
    if has_instruction {
        lines.push("- Pay special attention to the Repository Instruction. These contain important context and guidelines specific to this project.");
    }

    lines.join("\n")
}

/// Generate notes about content processing
pub fn generate_summary_notes(config: &OutputContextConfig) -> String {
    let mut notes = vec![
        "- Some files may have been excluded based on .gitignore rules and Repomix's configuration".to_string(),
        "- Binary files are not included in this packed representation. Please refer to the Repository Structure section for a complete list of file paths, including binary files".to_string(),
    ];

    // File selection notes
    if !config.include_patterns.is_empty() {
        notes.push(format!(
            "- Only files matching these patterns are included: {}",
            config.include_patterns.join(", ")
        ));
    }
    if !config.ignore_patterns.is_empty() {
        notes.push(format!(
            "- Files matching these patterns are excluded: {}",
            config.ignore_patterns.join(", ")
        ));
    }
    if config.use_gitignore {
        notes.push("- Files matching patterns in .gitignore are excluded".to_string());
    }
    if config.use_default_patterns {
        notes.push("- Files matching default ignore patterns are excluded".to_string());
    }

    // Processing notes
    if config.remove_comments {
        notes.push("- Code comments have been removed from supported file types".to_string());
    }
    if config.remove_empty_lines {
        notes.push("- Empty lines have been removed from all files".to_string());
    }
    if config.show_line_numbers {
        notes.push("- Line numbers have been added to the beginning of each line".to_string());
    }
    if config.compress {
        notes.push("- Content has been compressed - code blocks are separated by ⋮---- delimiter".to_string());
    }
    if config.truncate_base64 {
        notes.push("- Long base64 data strings (e.g., data:image/png;base64,...) have been truncated to reduce token count".to_string());
    }
    if !config.security_check {
        notes.push("- Security check has been disabled - content may contain sensitive information".to_string());
    }

    notes.join("\n")
}

/// Calculate markdown code block delimiter
/// Finds the longest sequence of backticks in files and returns one more
pub fn calculate_markdown_delimiter(files: &[ProcessedFile]) -> String {
    let max_backticks = files
        .iter()
        .flat_map(|file| {
            file.content
                .match_indices('`')
                .fold(Vec::new(), |mut acc, (i, _)| {
                    // Count consecutive backticks
                    let count = file.content[i..].chars().take_while(|c| *c == '`').count();
                    if count > 0 {
                        acc.push(count);
                    }
                    acc
                })
        })
        .max()
        .unwrap_or(0);

    "`".repeat(std::cmp::max(3, max_backticks + 1))
}

/// Get language name from file extension for syntax highlighting
pub fn get_language_from_extension(path: &str) -> &'static str {
    let ext = path
        .rsplit('.')
        .next()
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        // JavaScript/TypeScript
        "js" | "mjs" | "cjs" | "jsx" => "javascript",
        "ts" | "mts" | "cts" | "tsx" => "typescript",
        // Web frameworks
        "vue" => "vue",
        "svelte" => "svelte",
        "astro" => "astro",
        // Python
        "py" | "pyw" | "pyi" => "python",
        // Ruby
        "rb" => "ruby",
        "erb" => "erb",
        // Java/JVM
        "java" => "java",
        "kt" | "kts" => "kotlin",
        "scala" => "scala",
        "groovy" => "groovy",
        "clj" | "cljs" | "cljc" => "clojure",
        // C/C++
        "c" | "h" => "c",
        "cpp" | "cc" | "cxx" | "hpp" | "hxx" => "cpp",
        "m" | "mm" => "objectivec",
        // C#/F#/.NET
        "cs" => "csharp",
        "fs" | "fsx" | "fsi" => "fsharp",
        "vb" => "vb",
        // Go
        "go" => "go",
        // Rust
        "rs" => "rust",
        // Swift
        "swift" => "swift",
        // PHP
        "php" => "php",
        // Dart
        "dart" => "dart",
        // Shell
        "sh" | "bash" => "bash",
        "zsh" => "zsh",
        "fish" => "fish",
        "ps1" | "psm1" => "powershell",
        "bat" | "cmd" => "batch",
        // Markup/Style
        "html" | "htm" | "xhtml" => "html",
        "css" => "css",
        "scss" => "scss",
        "sass" => "sass",
        "less" => "less",
        // Data formats
        "json" | "jsonc" => "json",
        "xml" | "xsl" | "xslt" | "svg" => "xml",
        "yaml" | "yml" => "yaml",
        "toml" => "toml",
        "ini" | "cfg" | "conf" => "ini",
        // Documentation
        "md" | "mdx" => "markdown",
        "rst" => "rst",
        "tex" | "latex" => "latex",
        // Database
        "sql" => "sql",
        "prisma" => "prisma",
        // DevOps
        "dockerfile" => "dockerfile",
        "tf" | "tfvars" | "hcl" => "hcl",
        "nix" => "nix",
        // Templates
        "hbs" | "handlebars" | "mustache" => "handlebars",
        "ejs" => "ejs",
        "jinja" | "jinja2" | "j2" => "jinja",
        "liquid" => "liquid",
        "pug" | "jade" => "pug",
        // Other
        "lua" => "lua",
        "r" => "r",
        "jl" => "julia",
        "vim" => "vim",
        "diff" | "patch" => "diff",
        "graphql" | "gql" => "graphql",
        "proto" => "protobuf",
        "sol" => "solidity",
        "hs" | "lhs" => "haskell",
        "ex" | "exs" => "elixir",
        "erl" | "hrl" => "erlang",
        "ml" | "mli" => "ocaml",
        "elm" => "elm",
        "nim" => "nim",
        "zig" => "zig",
        "v" => "v",
        "pl" | "pm" => "perl",
        "makefile" | "mk" => "makefile",
        "cmake" => "cmake",
        "glsl" | "vert" | "frag" => "glsl",
        "wgsl" => "wgsl",
        "asm" | "s" => "asm",
        _ => "",
    }
}

/// Generate output in the specified format
pub fn generate_output(
    files: &[CollectedFile],
    style: OutputStyle,
    config: &MergedConfig,
    instruction: Option<String>,
) -> String {
    let context = build_output_context(files, config, instruction);

    match style {
        OutputStyle::Xml => generate_xml(&context),
        OutputStyle::Markdown => generate_markdown(&context),
        OutputStyle::Json => generate_json(&context),
        OutputStyle::Plain => generate_plain(&context),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_header_entire_codebase() {
        let config = OutputContextConfig::default();
        let header = generate_header(&config);
        assert!(header.contains("entire codebase"));
        assert!(header.contains("Repomix"));
    }

    #[test]
    fn test_generate_header_with_includes() {
        let config = OutputContextConfig {
            include_patterns: vec!["src/**".to_string()],
            ..Default::default()
        };
        let header = generate_header(&config);
        assert!(header.contains("subset of the codebase"));
        assert!(header.contains("specifically included files"));
    }

    #[test]
    fn test_generate_header_with_processing() {
        let config = OutputContextConfig {
            remove_comments: true,
            compress: true,
            ..Default::default()
        };
        let header = generate_header(&config);
        assert!(header.contains("comments have been removed"));
        assert!(header.contains("compressed"));
    }

    #[test]
    fn test_get_language_from_extension() {
        assert_eq!(get_language_from_extension("main.rs"), "rust");
        assert_eq!(get_language_from_extension("app.ts"), "typescript");
        assert_eq!(get_language_from_extension("style.css"), "css");
        assert_eq!(get_language_from_extension("unknown.xyz"), "");
    }

    #[test]
    fn test_calculate_markdown_delimiter() {
        let files = vec![
            ProcessedFile {
                path: "test.md".to_string(),
                content: "```code```".to_string(),
            },
        ];
        let delimiter = calculate_markdown_delimiter(&files);
        assert!(delimiter.len() >= 4); // Should be at least 4 backticks
    }

    #[test]
    fn test_summary_file_format() {
        let format = generate_summary_file_format();
        assert!(format.contains("organized as follows"));
        assert!(format.contains("Directory structure"));
    }
}
