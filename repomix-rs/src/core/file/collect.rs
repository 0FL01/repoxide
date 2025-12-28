//! File collection and reading

use anyhow::Result;
use std::path::Path;

/// Collected file with content
#[derive(Debug)]
pub struct CollectedFile {
    pub path: String,
    pub content: String,
}

/// Collect and read files from paths
pub fn collect_files(_files: &[impl AsRef<Path>]) -> Result<Vec<CollectedFile>> {
    // Implementation will be added in Phase 3
    Ok(Vec::new())
}
