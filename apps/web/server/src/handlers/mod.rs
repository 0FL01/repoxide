//! HTTP handlers module

pub mod health;
pub mod pack;
pub mod upload;
pub mod web;

// Re-export handler functions for convenience
pub use health::health;
pub use pack::pack;
pub use upload::{assemble_chunks, upload_chunk, upload_init, upload_status};
pub use web::{index, index_ru, pack_page, schema_asset, site_fallback};
