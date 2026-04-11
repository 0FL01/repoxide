//! Markdown output format
//!
//! Generates Markdown output for repository contents.

use super::generate::{
    calculate_markdown_delimiter, generate_header, generate_summary_file_format,
    generate_summary_notes, generate_summary_purpose, generate_summary_usage_guidelines,
    get_language_from_extension, OutputContext,
};

/// Generate Markdown output
pub fn generate_markdown(context: &OutputContext) -> String {
    let mut output = String::new();

    // Calculate code block delimiter (accounting for files that might contain backticks)
    let delimiter = calculate_markdown_delimiter(&context.files);

    // File summary section
    if context.config.file_summary {
        let header = generate_header(&context.config);
        output.push_str(&header);
        output.push_str("\n\n");

        output.push_str("# File Summary\n\n");

        output.push_str("## Purpose\n");
        output.push_str(&generate_summary_purpose(&context.config));
        output.push_str("\n\n");

        output.push_str("## File Format\n");
        output.push_str(&generate_summary_file_format());
        output.push_str("\n5. Multiple file entries, each consisting of:\n");
        output.push_str("  a. A header with the file path (## File: path/to/file)\n");
        output.push_str("  b. The full contents of the file in a code block\n\n");

        output.push_str("## Usage Guidelines\n");
        output.push_str(&generate_summary_usage_guidelines(
            context.header_text.is_some(),
            context.instruction.is_some(),
        ));
        output.push_str("\n\n");

        output.push_str("## Notes\n");
        output.push_str(&generate_summary_notes(&context.config));
        output.push_str("\n\n");
    }

    // User provided header
    if let Some(ref header_text) = context.header_text {
        output.push_str("# User Provided Header\n");
        output.push_str(header_text);
        output.push_str("\n\n");
    }

    // Directory structure
    if context.config.directory_structure {
        output.push_str("# Directory Structure\n");
        output.push_str("```\n");
        output.push_str(&context.tree_string);
        output.push_str("\n```\n\n");
    }

    // Files section
    if context.config.files {
        output.push_str("# Files\n\n");

        for file in &context.files {
            output.push_str(&format!("## File: {}\n", file.path));

            // Get language for syntax highlighting
            let lang = get_language_from_extension(&file.path);
            output.push_str(&delimiter);
            output.push_str(lang);
            output.push('\n');
            output.push_str(&file.content);
            output.push('\n');
            output.push_str(&delimiter);
            output.push_str("\n\n");
        }
    }

    // Instruction section
    if let Some(ref instruction) = context.instruction {
        output.push_str("# Instruction\n");
        output.push_str(instruction);
        output.push('\n');
    }

    output.trim_end().to_string() + "\n"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::output::generate::{OutputContextConfig, ProcessedFile};

    fn create_test_context() -> OutputContext {
        OutputContext {
            tree_string: "src/\n  main.rs\n  lib.rs".to_string(),
            files: vec![
                ProcessedFile {
                    path: "src/main.rs".to_string(),
                    content: "fn main() {\n    println!(\"Hello\");\n}".to_string(),
                },
                ProcessedFile {
                    path: "src/lib.rs".to_string(),
                    content: "pub mod test;".to_string(),
                },
            ],

            config: OutputContextConfig::default(),
            instruction: None,
            header_text: None,
        }
    }

    #[test]
    fn test_generate_markdown_basic() {
        let context = create_test_context();
        let output = generate_markdown(&context);

        assert!(output.contains("# File Summary"));
        assert!(output.contains("# Directory Structure"));
        assert!(output.contains("# Files"));
        assert!(output.contains("## File: src/main.rs"));
        assert!(output.contains("```rust"));
        assert!(output.contains("fn main()"));
    }

    #[test]
    fn test_generate_markdown_with_header() {
        let mut context = create_test_context();
        context.header_text = Some("This is my project".to_string());

        let output = generate_markdown(&context);

        assert!(output.contains("# User Provided Header"));
        assert!(output.contains("This is my project"));
    }

    #[test]
    fn test_generate_markdown_with_instruction() {
        let mut context = create_test_context();
        context.instruction = Some("Please review this code".to_string());

        let output = generate_markdown(&context);

        assert!(output.contains("# Instruction"));
        assert!(output.contains("Please review this code"));
    }

    #[test]
    fn test_generate_markdown_no_file_summary() {
        let mut context = create_test_context();
        context.config.file_summary = false;

        let output = generate_markdown(&context);

        assert!(!output.contains("# File Summary"));
        assert!(output.contains("# Files"));
    }

    #[test]
    fn test_generate_markdown_language_detection() {
        let mut context = create_test_context();
        context.files = vec![
            ProcessedFile {
                path: "app.ts".to_string(),
                content: "console.log('hello');".to_string(),
            },
            ProcessedFile {
                path: "style.css".to_string(),
                content: "body { margin: 0; }".to_string(),
            },
        ];

        let output = generate_markdown(&context);

        assert!(output.contains("```typescript"));
        assert!(output.contains("```css"));
    }

    #[test]
    fn test_generate_markdown_backticks_escape() {
        let mut context = create_test_context();
        context.files = vec![ProcessedFile {
            path: "readme.md".to_string(),
            content: "```rust\nfn main() {}\n```".to_string(),
        }];

        let output = generate_markdown(&context);

        // Should use more backticks than the content contains
        assert!(output.contains("````")); // At least 4 backticks
    }
}
