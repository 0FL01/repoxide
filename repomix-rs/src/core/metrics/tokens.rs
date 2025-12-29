//! Token counting using tiktoken-rs
//!
//! Uses o200k_base encoding (GPT-4o and newer models)

use std::sync::OnceLock;
use rayon::prelude::*;
use tiktoken_rs::{o200k_base, CoreBPE};

/// Global singleton for the tokenizer (BPE encoding is expensive to initialize)
static TOKENIZER: OnceLock<CoreBPE> = OnceLock::new();

/// Get or initialize the global tokenizer
fn get_tokenizer() -> &'static CoreBPE {
    TOKENIZER.get_or_init(|| {
        o200k_base().expect("Failed to initialize o200k_base tokenizer")
    })
}

/// Count tokens in text using o200k_base encoding
///
/// # Arguments
/// * `text` - The text to count tokens for
///
/// # Returns
/// The number of tokens in the text. Returns 0 if encoding fails.
pub fn count_tokens(text: &str) -> usize {
    let tokenizer = get_tokenizer();
    tokenizer.encode_ordinary(text).len()
}



/// Metrics result for a single file
#[derive(Debug, Clone)]
pub struct FileMetrics {
    /// File path (relative to root)
    pub path: String,
    /// Character count

    /// Token count
    pub tokens: usize,
}

/// Aggregate metrics for all files
#[derive(Debug, Clone, Default)]
pub struct PackMetrics {
    /// Total number of files
    pub total_files: usize,
    /// Total character count (in output)
    pub total_characters: usize,
    /// Total token count (in output)
    pub total_tokens: usize,
    /// Per-file character counts
    pub file_char_counts: Vec<FileMetrics>,
}

impl PackMetrics {
    /// Create new metrics with calculated values
    pub fn calculate(
        file_contents: &[(String, String)], // (path, content) pairs
        output: &str,
    ) -> Self {
        let total_files = file_contents.len();
        let total_characters = output.len();
        let total_tokens = count_tokens(output);

        // Calculate file metrics in parallel using Rayon
        let mut file_char_counts: Vec<FileMetrics> = file_contents
            .par_iter()
            .map(|(path, content)| FileMetrics {
                path: path.clone(),
                // characters: content.len(),
                tokens: count_tokens(content),
            })
            .collect();

        // Sort by token count descending
        file_char_counts.par_sort_by(|a, b| b.tokens.cmp(&a.tokens));

        PackMetrics {
            total_files,
            total_characters,
            total_tokens,
            file_char_counts,
        }
    }

    /// Get top N files by token count
    pub fn top_files(&self, n: usize) -> &[FileMetrics] {
        let len = self.file_char_counts.len().min(n);
        &self.file_char_counts[..len]
    }


}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_tokens_empty() {
        let count = count_tokens("");
        assert_eq!(count, 0);
    }

    #[test]
    fn test_count_tokens_simple() {
        let count = count_tokens("Hello, world!");
        // Token count should be > 0 for non-empty text
        assert!(count > 0);
        // "Hello, world!" typically tokenizes to a few tokens
        assert!(count < 10);
    }

    #[test]
    fn test_count_tokens_code() {
        let code = r#"
fn main() {
    println!("Hello, world!");
}
"#;
        let count = count_tokens(code);
        assert!(count > 5);
    }

    #[test]
    fn test_count_tokens_unicode() {
        let unicode_text = "Привет, мир! 你好世界 🌍";
        let count = count_tokens(unicode_text);
        assert!(count > 0);
    }



    #[test]
    fn test_file_metrics() {
        let files = vec![
            ("file1.rs".to_string(), "fn main() {}".to_string()),
            ("file2.rs".to_string(), "let x = 1;".to_string()),
        ];
        let output = "combined output";
        
        let metrics = PackMetrics::calculate(&files, output);
        
        assert_eq!(metrics.total_files, 2);
        assert!(metrics.total_tokens > 0);
        assert_eq!(metrics.file_char_counts.len(), 2);
    }

    #[test]
    fn test_top_files() {
        let files = vec![
            ("small.rs".to_string(), "x".to_string()),
            ("large.rs".to_string(), "fn main() { println!(\"Hello, world!\"); }".to_string()),
            ("medium.rs".to_string(), "fn test() {}".to_string()),
        ];
        let output = "output";
        
        let metrics = PackMetrics::calculate(&files, output);
        let top = metrics.top_files(2);
        
        assert_eq!(top.len(), 2);
        // Should be sorted by token count descending
        assert!(top[0].tokens >= top[1].tokens);
    }


}
