//! Output generation module
//!
//! Provides output generation in multiple formats: XML, Markdown, JSON, and Plain text.

pub mod generate;
pub mod json;
pub mod markdown;
pub mod plain;
pub mod xml;

pub use generate::{
    build_output_context, generate_output, get_language_from_extension, OutputContext,
    OutputContextConfig, ProcessedFile,
};
pub use json::generate_json;
