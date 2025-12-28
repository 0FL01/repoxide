//! Repomix - Pack repository contents to single file for AI consumption
//!
//! This is the main entry point for the repomix CLI application.

use anyhow::Result;

mod cli;
mod config;
mod core;
mod remote;
mod shared;

fn main() -> Result<()> {
    // Run the CLI application
    cli::run()
}
