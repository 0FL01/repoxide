//! Code compression module using tree-sitter

pub mod languages;
pub mod parser;

pub use parser::compress_code;
