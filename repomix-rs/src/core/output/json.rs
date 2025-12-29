//! JSON output format
//!
//! Generates JSON output for repository contents.

use serde::Serialize;

use super::generate::{
    generate_header, generate_summary_notes, generate_summary_purpose,
    generate_summary_usage_guidelines, OutputContext,
};

/// JSON file summary structure
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FileSummary {
    generation_header: String,
    purpose: String,
    file_format: String,
    usage_guidelines: String,
    notes: String,
}

/// JSON output document structure
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct JsonOutput {
    #[serde(skip_serializing_if = "Option::is_none")]
    file_summary: Option<FileSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_provided_header: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    directory_structure: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    files: Option<std::collections::HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    instruction: Option<String>,
}

/// Generate JSON file format description
fn generate_json_file_format() -> String {
    "The content is organized as follows:\n\
     1. This summary section\n\
     2. Repository information\n\
     3. Directory structure\n\
     4. Repository files, each consisting of:\n\
        - File path as a key\n\
        - Full contents of the file as the value"
        .to_string()
}

/// Generate JSON output
pub fn generate_json(context: &OutputContext) -> String {
    // Build file summary
    let file_summary = if context.config.file_summary {
        Some(FileSummary {
            generation_header: generate_header(&context.config),
            purpose: generate_summary_purpose(&context.config),
            file_format: generate_json_file_format(),
            usage_guidelines: generate_summary_usage_guidelines(
                context.header_text.is_some(),
                context.instruction.is_some(),
            ),
            notes: generate_summary_notes(&context.config),
        })
    } else {
        None
    };

    // Build files map
    let files = if context.config.files {
        let mut map = std::collections::HashMap::new();
        for file in &context.files {
            map.insert(file.path.clone(), file.content.clone());
        }
        Some(map)
    } else {
        None
    };

    // Build directory structure
    let directory_structure = if context.config.directory_structure {
        Some(context.tree_string.clone())
    } else {
        None
    };

    // Build output document
    let output = JsonOutput {
        file_summary,
        user_provided_header: context.header_text.clone(),
        directory_structure,
        files,
        instruction: context.instruction.clone(),
    };

    // Serialize with pretty printing
    serde_json::to_string_pretty(&output).unwrap_or_else(|e| {
        format!("{{\"error\": \"Failed to generate JSON: {}\"}}", e)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::output::generate::{OutputContextConfig, ProcessedFile};
    use std::collections::HashMap;

    fn create_test_context() -> OutputContext {
        OutputContext {
            generation_date: "2025-01-01T00:00:00Z".to_string(),
            tree_string: "src/\n  main.rs\n  lib.rs".to_string(),
            files: vec![
                ProcessedFile {
                    path: "src/main.rs".to_string(),
                    content: "fn main() {}".to_string(),
                },
                ProcessedFile {
                    path: "src/lib.rs".to_string(),
                    content: "pub mod test;".to_string(),
                },
            ],
            file_line_counts: HashMap::new(),
            config: OutputContextConfig::default(),
            instruction: None,
            header_text: None,
        }
    }

    #[test]
    fn test_generate_json_basic() {
        let context = create_test_context();
        let output = generate_json(&context);

        // Verify it's valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

        assert!(parsed.get("fileSummary").is_some());
        assert!(parsed.get("directoryStructure").is_some());
        assert!(parsed.get("files").is_some());

        // Check files are present
        let files = parsed.get("files").unwrap().as_object().unwrap();
        assert!(files.contains_key("src/main.rs"));
        assert!(files.contains_key("src/lib.rs"));
    }

    #[test]
    fn test_generate_json_with_header() {
        let mut context = create_test_context();
        context.header_text = Some("This is my project".to_string());

        let output = generate_json(&context);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

        assert_eq!(
            parsed.get("userProvidedHeader").unwrap().as_str().unwrap(),
            "This is my project"
        );
    }

    #[test]
    fn test_generate_json_with_instruction() {
        let mut context = create_test_context();
        context.instruction = Some("Please review this code".to_string());

        let output = generate_json(&context);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

        assert_eq!(
            parsed.get("instruction").unwrap().as_str().unwrap(),
            "Please review this code"
        );
    }

    #[test]
    fn test_generate_json_no_file_summary() {
        let mut context = create_test_context();
        context.config.file_summary = false;

        let output = generate_json(&context);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

        assert!(parsed.get("fileSummary").is_none());
        assert!(parsed.get("files").is_some());
    }

    #[test]
    fn test_generate_json_structure() {
        let context = create_test_context();
        let output = generate_json(&context);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

        // Check file summary structure
        let summary = parsed.get("fileSummary").unwrap();
        assert!(summary.get("generationHeader").is_some());
        assert!(summary.get("purpose").is_some());
        assert!(summary.get("fileFormat").is_some());
        assert!(summary.get("usageGuidelines").is_some());
        assert!(summary.get("notes").is_some());
    }
}
