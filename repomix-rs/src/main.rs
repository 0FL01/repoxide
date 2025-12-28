//! Repomix - Pack repository contents to single file for AI consumption

use anyhow::Result;

mod cli;
mod config;
mod core;
mod remote;
mod shared;

fn main() -> Result<()> {
    // Entry point will be implemented in Phase 2
    println!("Repomix v{}", env!("CARGO_PKG_VERSION"));
    Ok(())
}
