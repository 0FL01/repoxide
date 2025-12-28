//! Configuration file loader

use anyhow::{Context, Result};
use std::path::Path;

use super::schema::RepomixConfig;

const CONFIG_FILE_NAMES: &[&str] = &[
    "repomix.config.json",
    "repomix.config.jsonc",
    "repomix.config.json5",
];

/// Load configuration from a directory
pub fn load_config(directory: &Path) -> Result<RepomixConfig> {
    for config_name in CONFIG_FILE_NAMES {
        let config_path = directory.join(config_name);
        if config_path.exists() {
            return load_config_file(&config_path);
        }
    }
    
    // Return default config if no config file found
    Ok(RepomixConfig::default())
}

/// Load configuration from a specific file
fn load_config_file(path: &Path) -> Result<RepomixConfig> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {:?}", path))?;
    
    // Parse JSON (basic implementation, JSON5 support can be added later)
    let config: RepomixConfig = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse config file: {:?}", path))?;
    
    Ok(config)
}
