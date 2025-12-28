//! CLI module - Command line interface and argument parsing
//!
//! This module provides the command line interface for repomix.

pub mod args;
pub mod run;

pub use args::{Args, Command, OutputStyle};
pub use run::{run, CliContext, LogLevel};
