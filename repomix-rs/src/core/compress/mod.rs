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
//! use repomix::core::compress::compress_code;
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

pub use languages::{get_language_from_extension, SupportedLanguage};
pub use parser::{compress_code, CHUNK_SEPARATOR};
