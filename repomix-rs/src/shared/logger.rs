//! Logging utilities

use colored::Colorize;

/// Log level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

/// Log an info message
pub fn info(message: &str) {
    println!("{} {}", "ℹ".blue(), message);
}

/// Log a success message
pub fn success(message: &str) {
    println!("{} {}", "✓".green(), message);
}

/// Log a warning message
pub fn warn(message: &str) {
    println!("{} {}", "⚠".yellow(), message);
}

/// Log an error message
pub fn error(message: &str) {
    eprintln!("{} {}", "✗".red(), message);
}
