//! Tree-sitter parsing and code compression
//!
//! This module implements the `--compress` functionality that extracts
//! function/class signatures from source code using tree-sitter parsing.

use arborium_tree_sitter::{Parser, Query, QueryCursor};
use std::collections::HashSet;
use std::path::Path;

use super::languages::{get_language_from_extension, SupportedLanguage};
use super::strategies::{
    CStyleStrategy, GoStrategy, LanguageStrategy, PythonStrategy, RubyStrategy, TypeScriptStrategy,
};

/// The chunk separator used in compressed output
pub const CHUNK_SEPARATOR: &str = "⋮----";

/// Represents a captured chunk of code
#[derive(Debug, Clone)]
struct CapturedChunk {
    content: String,
    start_row: usize,
    end_row: usize,
}

/// Compress code using tree-sitter to extract function/class signatures
///
/// Returns None if the language is not supported or parsing fails.
/// Returns Some(compressed_content) on success.
pub fn compress_code(content: &str, file_path: &str) -> Option<String> {
    // Get file extension
    let extension = Path::new(file_path).extension().and_then(|e| e.to_str())?;

    // Get language from extension
    let language = get_language_from_extension(extension)?;

    // Parse the file
    parse_file(content, language)
}

/// Parse file content and extract signatures
fn parse_file(content: &str, language: SupportedLanguage) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return Some(String::new());
    }

    // Create parser
    let mut parser = Parser::new();
    let ts_language = language.get_ts_language();
    parser.set_language(&ts_language).ok()?;

    // Parse the content
    let tree = parser.parse(content, None)?;

    // Create query - handle potential query errors gracefully
    let query_source = language.get_query();
    let query = match Query::new(&ts_language, query_source) {
        Ok(q) => q,
        Err(e) => {
            eprintln!("Query error for {}: {:?}", language.name(), e);
            return None;
        }
    };

    // Execute query using matches with StreamingIterator
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), content.as_bytes());

    let mut processed_chunks: HashSet<String> = HashSet::new();
    let mut captured_chunks: Vec<CapturedChunk> = Vec::new();

    // Select strategy
    let strategy: Box<dyn LanguageStrategy> = match language {
        SupportedLanguage::Python => Box::new(PythonStrategy),
        SupportedLanguage::Ruby => Box::new(RubyStrategy),
        SupportedLanguage::TypeScript | SupportedLanguage::JavaScript => {
            Box::new(TypeScriptStrategy)
        }
        SupportedLanguage::Go => Box::new(GoStrategy),
        _ => Box::new(CStyleStrategy),
    };

    // Process all matches using StreamingIterator
    use streaming_iterator::StreamingIterator;
    while let Some(query_match) = matches.next() {
        for capture in query_match.captures {
            let node = capture.node;
            let capture_name = query.capture_names()[capture.index as usize];
            let start_row = node.start_position().row;
            let end_row = node.end_position().row;

            // Only process certain capture types
            if should_capture(capture_name) {
                if let Some(chunk_content) = extract_chunk(
                    &lines,
                    start_row,
                    end_row,
                    capture_name,
                    &mut processed_chunks,
                    &*strategy,
                ) {
                    captured_chunks.push(CapturedChunk {
                        content: chunk_content.trim().to_string(),
                        start_row,
                        end_row,
                    });
                }
            }
        }
    }

    // Filter and merge chunks
    let filtered_chunks = filter_duplicated_chunks(captured_chunks);
    let merged_chunks = merge_adjacent_chunks(filtered_chunks);

    // Join chunks with separator
    let result = merged_chunks
        .into_iter()
        .map(|c| c.content)
        .collect::<Vec<_>>()
        .join(&format!("\n{}\n", CHUNK_SEPARATOR));

    Some(result.trim().to_string())
}

/// Determine if a capture name should be processed
fn should_capture(capture_name: &str) -> bool {
    // Process all definition and name captures
    capture_name.contains("definition")
        || capture_name.contains("name")
        || capture_name.contains("comment")
        || capture_name.contains("import")
}

/// Extract chunk content from lines
fn extract_chunk(
    lines: &[&str],
    start_row: usize,
    end_row: usize,
    capture_name: &str,
    processed_chunks: &mut HashSet<String>,
    strategy: &dyn LanguageStrategy,
) -> Option<String> {
    if start_row >= lines.len() {
        return None;
    }

    let actual_end = end_row.min(lines.len().saturating_sub(1));

    // For function/method definitions, try to extract just the signature
    if capture_name.contains("function") || capture_name.contains("method") {
        if let Some(signature) = strategy.extract_signature(lines, start_row, actual_end) {
            let normalized = signature.trim().to_string();
            if !processed_chunks.contains(&normalized) {
                processed_chunks.insert(normalized.clone());
                return Some(signature);
            }
            return None;
        }
    }

    // For class/interface definitions, extract the declaration line(s)
    if capture_name.contains("class") || capture_name.contains("interface") {
        if let Some(declaration) = strategy.extract_declaration(lines, start_row, actual_end) {
            let normalized = declaration.trim().to_string();
            if !processed_chunks.contains(&normalized) {
                processed_chunks.insert(normalized.clone());
                return Some(declaration);
            }
            return None;
        }
    }

    // For other captures (imports, comments, etc.), use full content
    let selected_lines: Vec<&str> = lines[start_row..=actual_end].to_vec();
    let chunk = selected_lines.join("\n");
    let normalized = chunk.trim().to_string();

    if !processed_chunks.contains(&normalized) {
        processed_chunks.insert(normalized);
        return Some(chunk);
    }

    None
}

/// Filter out duplicated chunks (keep the longest one for each start row)
fn filter_duplicated_chunks(chunks: Vec<CapturedChunk>) -> Vec<CapturedChunk> {
    use std::collections::HashMap;

    // Group chunks by start row
    let mut by_start_row: HashMap<usize, Vec<CapturedChunk>> = HashMap::new();
    for chunk in chunks {
        by_start_row.entry(chunk.start_row).or_default().push(chunk);
    }

    // Keep the chunk with most content for each start row
    let mut filtered: Vec<CapturedChunk> = by_start_row
        .into_values()
        .map(|mut row_chunks| {
            row_chunks.sort_by(|a, b| b.content.len().cmp(&a.content.len()));
            row_chunks.remove(0)
        })
        .collect();

    // Sort by start row
    filtered.sort_by_key(|c| c.start_row);
    filtered
}

/// Merge adjacent chunks (consecutive lines)
fn merge_adjacent_chunks(chunks: Vec<CapturedChunk>) -> Vec<CapturedChunk> {
    if chunks.len() <= 1 {
        return chunks;
    }

    let mut merged: Vec<CapturedChunk> = Vec::new();
    let mut iter = chunks.into_iter();

    if let Some(first) = iter.next() {
        merged.push(first);
    }

    for current in iter {
        let last = merged.last_mut().unwrap();

        // Merge if adjacent (end_row + 1 == start_row)
        if last.end_row + 1 == current.start_row {
            last.content = format!("{}\n{}", last.content, current.content);
            last.end_row = current.end_row;
        } else {
            merged.push(current);
        }
    }

    merged
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_unsupported_language() {
        let result = compress_code("some content", "file.xyz");
        assert!(result.is_none());
    }

    #[test]
    fn test_compress_empty_content() {
        let result = compress_code("", "file.rs");
        if let Some(s) = result {
            assert!(s.is_empty());
        }
    }

    #[test]
    fn test_compress_rust_function() {
        let content = r#"
fn hello_world() {
    println!("Hello, world!");
}
"#;
        let result = compress_code(content, "test.rs");
        assert!(result.is_some());
        let compressed = result.unwrap();
        assert!(compressed.contains("fn hello_world()"));
        // Should not contain the function body
        assert!(!compressed.contains("println!"));
    }

    #[test]
    fn test_compress_rust_struct() {
        let content = r#"
pub struct MyStruct {
    field1: i32,
    field2: String,
}
"#;
        let result = compress_code(content, "test.rs");
        assert!(result.is_some());
        let compressed = result.unwrap();
        assert!(compressed.contains("struct MyStruct"));
    }

    #[test]
    fn test_compress_python_function() {
        let content = r#"
def hello():
    print("Hello")

class MyClass:
    def method(self):
        pass
"#;
        let result = compress_code(content, "test.py");
        assert!(result.is_some());
        let compressed = result.unwrap();
        assert!(compressed.contains("def hello():"));
        assert!(compressed.contains("class MyClass:"));
        // Ensure body is not captured
        assert!(!compressed.contains("print(\"Hello\")"));
        assert!(!compressed.contains("pass"));
    }

    #[test]
    fn test_compress_javascript_function() {
        let content = r#"
function hello() {
    console.log("hello");
}

const arrowFn = () => {
    return 42;
};

class MyClass {
    constructor() {}
    method() {}
}
"#;
        let result = compress_code(content, "test.js");
        assert!(result.is_some());
        let compressed = result.unwrap();
        assert!(compressed.contains("function hello"));
        assert!(compressed.contains("class MyClass"));
    }

    #[test]
    fn test_compress_typescript_interface() {
        let content = r#"
interface User {
    name: string;
    age: number;
}

type Status = "active" | "inactive";

class UserService {
    getUser(id: string): User {
        return { name: "test", age: 25 };
    }
}
"#;
        let result = compress_code(content, "test.ts");
        assert!(result.is_some());
        let compressed = result.unwrap();
        assert!(compressed.contains("interface User"));
        assert!(compressed.contains("class UserService"));
    }

    #[test]
    fn test_compress_ruby_method() {
        let content = r#"
def hello(name)
  puts "Hello #{name}"
end

class MyClass
  def method
    true
  end
end
"#;
        let result = compress_code(content, "test.rb");
        assert!(result.is_some());
        let compressed = result.unwrap();
        // It should contain the method name and arguments
        assert!(compressed.contains("def hello(name)"));
        assert!(compressed.contains("def method"));

        // It SHOULD NOT contain the method body if it works correctly
        assert!(!compressed.contains("puts \"Hello"));
        assert!(!compressed.contains("true"));
    }

    #[test]
    fn test_compress_go_function() {
        let content = r#"
package main

import "fmt"

func main() {
    fmt.Println("hello")
}

type MyStruct struct {
    Field int
}

func (s *MyStruct) Method(x int) bool {
    return x > 0
}
"#;
        let result = compress_code(content, "test.go");
        assert!(result.is_some());
        let compressed = result.unwrap();
        assert!(compressed.contains("func main()"));
        assert!(compressed.contains("type MyStruct struct"));
        assert!(compressed.contains("func (s *MyStruct) Method(x int) bool"));

        // Body should be excluded
        assert!(!compressed.contains("fmt.Println"));
        assert!(!compressed.contains("return x > 0"));
    }

    #[test]
    fn test_compress_swift_function() {
        let content = r#"
import Foundation

class MyClass {
    func hello(name: String) -> String {
        return "Hello \(name)"
    }
}
"#;
        let result = compress_code(content, "test.swift");
        assert!(result.is_some());
        let compressed = result.unwrap();
        assert!(compressed.contains("class MyClass"));
        assert!(compressed.contains("func hello(name: String) -> String"));
        assert!(!compressed.contains("return \"Hello"));
    }

    #[test]
    fn test_compress_c_function_brace_on_next_line() {
        let content = r#"
void foo()
{
    printf("hello");
}
"#;
        let result = compress_code(content, "test.c");
        assert!(result.is_some());
        let compressed = result.unwrap();
        // Should contain void foo()
        assert!(compressed.contains("void foo()"));
        // Should NOT contain the brace {
        assert!(!compressed.contains('{'));
        // Should NOT contain the body
        assert!(!compressed.contains("printf"));
    }

    #[test]
    fn test_chunk_separator_constant() {
        assert_eq!(CHUNK_SEPARATOR, "⋮----");
    }

    #[test]
    fn test_should_capture() {
        assert!(should_capture("definition.function"));
        assert!(should_capture("name.definition.class"));
        assert!(should_capture("definition.import"));
        assert!(should_capture("comment"));
        assert!(!should_capture("reference.call"));
        assert!(!should_capture("random"));
    }

    #[test]
    fn test_filter_duplicated_chunks() {
        let chunks = vec![
            CapturedChunk {
                content: "short".to_string(),
                start_row: 0,
                end_row: 0,
            },
            CapturedChunk {
                content: "longer content".to_string(),
                start_row: 0,
                end_row: 1,
            },
            CapturedChunk {
                content: "other".to_string(),
                start_row: 5,
                end_row: 5,
            },
        ];

        let filtered = filter_duplicated_chunks(chunks);
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].content, "longer content");
        assert_eq!(filtered[1].content, "other");
    }

    #[test]
    fn test_merge_adjacent_chunks() {
        let chunks = vec![
            CapturedChunk {
                content: "line1".to_string(),
                start_row: 0,
                end_row: 0,
            },
            CapturedChunk {
                content: "line2".to_string(),
                start_row: 1,
                end_row: 1,
            },
            CapturedChunk {
                content: "line5".to_string(),
                start_row: 5,
                end_row: 5,
            },
        ];

        let merged = merge_adjacent_chunks(chunks);
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0].content, "line1\nline2");
        assert_eq!(merged[1].content, "line5");
    }
}
