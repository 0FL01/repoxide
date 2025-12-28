//! Language detection and mapping

/// Get language name from file extension
pub fn get_language_from_extension(extension: &str) -> Option<&'static str> {
    match extension {
        "rs" => Some("rust"),
        "ts" | "tsx" => Some("typescript"),
        "js" | "jsx" | "mjs" => Some("javascript"),
        "py" => Some("python"),
        "go" => Some("go"),
        "java" => Some("java"),
        "c" | "h" => Some("c"),
        "cpp" | "hpp" | "cc" => Some("cpp"),
        "cs" => Some("csharp"),
        "rb" => Some("ruby"),
        "php" => Some("php"),
        "swift" => Some("swift"),
        "dart" => Some("dart"),
        "sol" => Some("solidity"),
        "css" => Some("css"),
        "vue" => Some("vue"),
        _ => None,
    }
}
