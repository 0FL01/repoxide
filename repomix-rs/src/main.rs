//! Repomix - Pack repository contents to single file for AI consumption
//!
//! This is the main entry point for the repomix CLI application.

use anyhow::Result;
use repomix::cli;

fn main() -> Result<()> {
    // Run the CLI application
    cli::run()
}
