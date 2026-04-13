//! Code compression module using tree-sitter
//!
//! This module provides functionality to compress source code by extracting
//! only the essential signatures (functions, classes, interfaces, etc.)
//! using tree-sitter parsing.
//!
//! # Supported Languages
//!
//! - Rust (.rs)
//! - TypeScript (.ts, .tsx, .mts, .mtsx, .cts)
//! - JavaScript (.js, .jsx, .cjs, .mjs, .mjsx)
//! - Python (.py)
//! - Go (.go)
//! - Java (.java)
//! - C (.c, .h)
//! - C++ (.cpp, .hpp, .cc, .cxx, .hxx)
//! - C# (.cs)
//! - Ruby (.rb)
//! - PHP (.php)
//! - CSS (.css)
//!
//! # Example
//!
//! ```rust,ignore
//! use repoxide::core::compress::compress_code;
//!
//! let rust_code = r#"
//! fn hello_world() {
//!     println!("Hello!");
//! }
//! "#;
//!
//! let compressed = compress_code(rust_code, "example.rs");
//! // Returns: Some("fn hello_world()")
//! ```

pub mod languages;
pub mod parser;
pub mod queries;
pub mod strategies;

use rayon::prelude::*;

use crate::core::file::CollectedFile;

pub use languages::{get_language_from_extension, SupportedLanguage};
pub use parser::{compress_code, CHUNK_SEPARATOR};

/// Compress files in place and return the number of files whose content changed.
pub fn compress_files_in_place(files: &mut [CollectedFile]) -> usize {
    files
        .par_iter_mut()
        .map(|file| {
            let original_len = file.content.len();

            if let Some(compressed) = compress_code(&file.content, &file.path) {
                if !compressed.is_empty() && compressed.len() < original_len {
                    file.content = compressed;
                    return 1;
                }
            }

            0
        })
        .sum()
}
