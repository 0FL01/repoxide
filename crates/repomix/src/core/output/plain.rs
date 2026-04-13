//! Plain text output format
//!
//! Generates plain text output for repository contents.

use super::generate::{
    generate_header, generate_summary_file_format, generate_summary_notes,
    generate_summary_purpose, generate_summary_usage_guidelines, OutputContext,
};

/// Short separator line
const SEPARATOR: &str = "================";

/// Long separator line
const LONG_SEPARATOR: &str = "================================================================";

/// Generate plain text output
pub fn generate_plain(context: &OutputContext<'_>) -> String {
    let mut output = String::new();

    // File summary section
    if context.config.file_summary {
        let header = generate_header(&context.config);
        output.push_str(&header);
        output.push_str("\n\n");

        output.push_str(LONG_SEPARATOR);
        output.push_str("\nFile Summary\n");
        output.push_str(LONG_SEPARATOR);
        output.push_str("\n\n");

        output.push_str("Purpose:\n");
        output.push_str("--------\n");
        output.push_str(&generate_summary_purpose(&context.config));
        output.push_str("\n\n");

        output.push_str("File Format:\n");
        output.push_str("------------\n");
        output.push_str(&generate_summary_file_format());
        output.push_str("\n5. Multiple file entries, each consisting of:\n");
        output.push_str("  a. A separator line (================)\n");
        output.push_str("  b. The file path (File: path/to/file)\n");
        output.push_str("  c. Another separator line\n");
        output.push_str("  d. The full contents of the file\n");
        output.push_str("  e. A blank line\n\n");

        output.push_str("Usage Guidelines:\n");
        output.push_str("-----------------\n");
        output.push_str(&generate_summary_usage_guidelines(
            context.header_text.is_some(),
            context.instruction.is_some(),
        ));
        output.push_str("\n\n");

        output.push_str("Notes:\n");
        output.push_str("------\n");
        output.push_str(&generate_summary_notes(&context.config));
        output.push_str("\n\n");
    }

    // User provided header
    if let Some(ref header_text) = context.header_text {
        output.push_str(LONG_SEPARATOR);
        output.push_str("\nUser Provided Header\n");
        output.push_str(LONG_SEPARATOR);
        output.push('\n');
        output.push_str(header_text);
        output.push_str("\n\n");
    }

    // Directory structure
    if context.config.directory_structure {
        output.push_str(LONG_SEPARATOR);
        output.push_str("\nDirectory Structure\n");
        output.push_str(LONG_SEPARATOR);
        output.push('\n');
        output.push_str(&context.tree_string);
        output.push_str("\n\n");
    }

    // Files section
    if context.config.files {
        output.push_str(LONG_SEPARATOR);
        output.push_str("\nFiles\n");
        output.push_str(LONG_SEPARATOR);
        output.push_str("\n\n");

        for file in &context.files {
            output.push_str(SEPARATOR);
            output.push_str(&format!("\nFile: {}\n", file.path));
            output.push_str(SEPARATOR);
            output.push('\n');
            output.push_str(&file.content);
            output.push_str("\n\n");
        }
    }

    // Instruction section
    if let Some(ref instruction) = context.instruction {
        output.push_str(LONG_SEPARATOR);
        output.push_str("\nInstruction\n");
        output.push_str(LONG_SEPARATOR);
        output.push('\n');
        output.push_str(instruction);
        output.push_str("\n\n");
    }

    // End marker
    output.push_str(LONG_SEPARATOR);
    output.push_str("\nEnd of Codebase\n");
    output.push_str(LONG_SEPARATOR);
    output.push('\n');

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::output::generate::{OutputContextConfig, ProcessedFile};

    fn create_test_context() -> OutputContext<'static> {
        OutputContext {
            tree_string: "src/\n  main.rs\n  lib.rs".to_string(),
            files: vec![
                ProcessedFile {
                    path: "src/main.rs".into(),
                    content: "fn main() {}".into(),
                },
                ProcessedFile {
                    path: "src/lib.rs".into(),
                    content: "pub mod test;".into(),
                },
            ],

            config: OutputContextConfig::default(),
            instruction: None,
            header_text: None,
        }
    }

    #[test]
    fn test_generate_plain_basic() {
        let context = create_test_context();
        let output = generate_plain(&context);

        assert!(output.contains("File Summary"));
        assert!(output.contains("Directory Structure"));
        assert!(output.contains("Files"));
        assert!(output.contains("File: src/main.rs"));
        assert!(output.contains("fn main()"));
        assert!(output.contains("End of Codebase"));
    }

    #[test]
    fn test_generate_plain_with_header() {
        let mut context = create_test_context();
        context.header_text = Some("This is my project".to_string());

        let output = generate_plain(&context);

        assert!(output.contains("User Provided Header"));
        assert!(output.contains("This is my project"));
    }

    #[test]
    fn test_generate_plain_with_instruction() {
        let mut context = create_test_context();
        context.instruction = Some("Please review this code".to_string());

        let output = generate_plain(&context);

        assert!(output.contains("Instruction"));
        assert!(output.contains("Please review this code"));
    }

    #[test]
    fn test_generate_plain_no_file_summary() {
        let mut context = create_test_context();
        context.config.file_summary = false;

        let output = generate_plain(&context);

        assert!(!output.contains("File Summary\n===="));
        assert!(output.contains("Files"));
    }

    #[test]
    fn test_generate_plain_separators() {
        let context = create_test_context();
        let output = generate_plain(&context);

        // Check that separators are present
        assert!(output.contains(LONG_SEPARATOR));
        assert!(output.contains(SEPARATOR));
    }

    #[test]
    fn test_generate_plain_ends_with_marker() {
        let context = create_test_context();
        let output = generate_plain(&context);

        assert!(output.ends_with(&format!("End of Codebase\n{}\n", LONG_SEPARATOR)));
    }
}
