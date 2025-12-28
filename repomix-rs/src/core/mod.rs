//! Core module - Main functionality

pub mod compress;
pub mod file;
pub mod metrics;
pub mod output;
pub mod packager;

pub use packager::pack;
