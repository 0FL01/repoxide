//! XML output format
//!
//! Generates XML output for repository contents.

use super::generate::{
    generate_header, generate_summary_file_format, generate_summary_notes,
    generate_summary_purpose, generate_summary_usage_guidelines, OutputContext,
};

/// Escape XML special characters
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Generate XML output
pub fn generate_xml(context: &OutputContext) -> String {
    let mut output = String::new();

    // File summary section
    if context.config.file_summary {
        let header = generate_header(&context.config);
        output.push_str(&header);
        output.push_str("\n\n");

        output.push_str("<file_summary>\n");
        output.push_str("This section contains a summary of this file.\n\n");

        output.push_str("<purpose>\n");
        output.push_str(&generate_summary_purpose(&context.config));
        output.push_str("\n</purpose>\n\n");

        output.push_str("<file_format>\n");
        output.push_str(&generate_summary_file_format());
        output.push_str("\n5. Multiple file entries, each consisting of:\n");
        output.push_str("  - File path as an attribute\n");
        output.push_str("  - Full contents of the file\n");
        output.push_str("</file_format>\n\n");

        output.push_str("<usage_guidelines>\n");
        output.push_str(&generate_summary_usage_guidelines(
            context.header_text.is_some(),
            context.instruction.is_some(),
        ));
        output.push_str("\n</usage_guidelines>\n\n");

        output.push_str("<notes>\n");
        output.push_str(&generate_summary_notes(&context.config));
        output.push_str("\n</notes>\n\n");

        output.push_str("</file_summary>\n\n");
    }

    // User provided header
    if let Some(ref header_text) = context.header_text {
        output.push_str("<user_provided_header>\n");
        output.push_str(header_text);
        output.push_str("\n</user_provided_header>\n\n");
    }

    // Directory structure
    if context.config.directory_structure {
        output.push_str("<directory_structure>\n");
        output.push_str(&context.tree_string);
        output.push_str("\n</directory_structure>\n\n");
    }

    // Files section
    if context.config.files {
        output.push_str("<files>\n");
        output.push_str("This section contains the contents of the repository's files.\n\n");

        for file in &context.files {
            // Escape the path for use in attribute
            let escaped_path = escape_xml(&file.path);
            output.push_str(&format!("<file path=\"{}\">\n", escaped_path));

            // Content - for XML we escape special characters if parsable_style is enabled
            if context.config.parsable_style {
                output.push_str(&escape_xml(&file.content));
            } else {
                output.push_str(&file.content);
            }

            output.push_str("\n</file>\n\n");
        }

        output.push_str("</files>\n");
    }

    // Instruction section
    if let Some(ref instruction) = context.instruction {
        output.push_str("\n<instruction>\n");
        output.push_str(instruction);
        output.push_str("\n</instruction>\n");
    }

    output
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
    fn test_generate_xml_basic() {
        let context = create_test_context();
        let output = generate_xml(&context);

        assert!(output.contains("<file_summary>"));
        assert!(output.contains("</file_summary>"));
        assert!(output.contains("<directory_structure>"));
        assert!(output.contains("<files>"));
        assert!(output.contains("<file path=\"src/main.rs\">"));
        assert!(output.contains("fn main()"));
    }

    #[test]
    fn test_generate_xml_with_header() {
        let mut context = create_test_context();
        context.header_text = Some("This is my project".to_string());

        let output = generate_xml(&context);

        assert!(output.contains("<user_provided_header>"));
        assert!(output.contains("This is my project"));
        assert!(output.contains("</user_provided_header>"));
    }

    #[test]
    fn test_generate_xml_with_instruction() {
        let mut context = create_test_context();
        context.instruction = Some("Please review this code".to_string());

        let output = generate_xml(&context);

        assert!(output.contains("<instruction>"));
        assert!(output.contains("Please review this code"));
        assert!(output.contains("</instruction>"));
    }

    #[test]
    fn test_generate_xml_no_file_summary() {
        let mut context = create_test_context();
        context.config.file_summary = false;

        let output = generate_xml(&context);

        assert!(!output.contains("<file_summary>"));
        assert!(output.contains("<files>"));
    }

    #[test]
    fn test_escape_xml() {
        assert_eq!(escape_xml("<test>"), "&lt;test&gt;");
        assert_eq!(escape_xml("a & b"), "a &amp; b");
        assert_eq!(escape_xml("\"quoted\""), "&quot;quoted&quot;");
    }

    #[test]
    fn test_generate_xml_parsable_style() {
        let mut context = create_test_context();
        context.config.parsable_style = true;
        context.files = vec![ProcessedFile {
            path: "test.txt".to_string(),
            content: "<script>alert('xss')</script>".to_string(),
        }];

        let output = generate_xml(&context);

        assert!(output.contains("&lt;script&gt;"));
        assert!(!output.contains("<script>"));
    }
}
