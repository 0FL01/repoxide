//! Configuration module - Schema and loading logic
//!
//! This module handles configuration file loading and merging.

pub mod loader;
pub mod schema;

pub use loader::load_config;
pub use schema::{
    GitOutputConfig, IgnoreConfig, InputConfig, MergedConfig, OutputConfig, 
    RepomixConfig, SecurityConfig, TokenCountConfig,
};
