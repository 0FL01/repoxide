//! Configuration module - Schema and loading logic

pub mod loader;
pub mod schema;

pub use loader::load_config;
pub use schema::RepomixConfig;
