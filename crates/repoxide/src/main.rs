//! Repoxide - Pack repository contents to single file for AI consumption
//!
//! This is the main entry point for the repoxide CLI application.

use anyhow::Result;
use repoxide::cli;

fn main() -> Result<()> {
    // Run the CLI application
    cli::run()
}
