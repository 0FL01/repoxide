//! Output generation module

pub mod generate;
pub mod json;
pub mod markdown;
pub mod plain;
pub mod xml;

pub use generate::generate_output;
