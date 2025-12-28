//! File handling module - Search, collect, and read files

pub mod collect;
pub mod search;
pub mod tree;

pub use collect::collect_files;
pub use search::search_files;
pub use tree::generate_tree;
