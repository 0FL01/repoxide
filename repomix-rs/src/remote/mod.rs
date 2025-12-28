//! Remote repository handling

pub mod clone;
pub mod parse;

pub use clone::clone_repository;
pub use parse::parse_remote_url;
