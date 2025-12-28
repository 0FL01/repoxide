//! Language detection, mapping, and tree-sitter parser initialization

use std::collections::HashMap;
use std::sync::OnceLock;
use tree_sitter::Language;

use super::queries;

/// Supported languages for compression
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SupportedLanguage {
    Rust,
    TypeScript,
    JavaScript,
    Python,
    Go,
    Java,
    C,
    Cpp,
    CSharp,
    Ruby,
    Php,
    Css,
}

impl SupportedLanguage {
    /// Get the tree-sitter Language for this language
    pub fn get_ts_language(&self) -> Language {
        match self {
            Self::Rust => tree_sitter_rust::LANGUAGE.into(),
            Self::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            Self::JavaScript => tree_sitter_javascript::LANGUAGE.into(),
            Self::Python => tree_sitter_python::LANGUAGE.into(),
            Self::Go => tree_sitter_go::LANGUAGE.into(),
            Self::Java => tree_sitter_java::LANGUAGE.into(),
            Self::C => tree_sitter_c::LANGUAGE.into(),
            Self::Cpp => tree_sitter_cpp::LANGUAGE.into(),
            Self::CSharp => tree_sitter_c_sharp::LANGUAGE.into(),
            Self::Ruby => tree_sitter_ruby::LANGUAGE.into(),
            Self::Php => tree_sitter_php::LANGUAGE_PHP.into(),
            Self::Css => tree_sitter_css::LANGUAGE.into(),
        }
    }

    /// Get the tree-sitter query string for this language
    pub fn get_query(&self) -> &'static str {
        match self {
            Self::Rust => queries::QUERY_RUST,
            Self::TypeScript => queries::QUERY_TYPESCRIPT,
            Self::JavaScript => queries::QUERY_JAVASCRIPT,
            Self::Python => queries::QUERY_PYTHON,
            Self::Go => queries::QUERY_GO,
            Self::Java => queries::QUERY_JAVA,
            Self::C => queries::QUERY_C,
            Self::Cpp => queries::QUERY_CPP,
            Self::CSharp => queries::QUERY_CSHARP,
            Self::Ruby => queries::QUERY_RUBY,
            Self::Php => queries::QUERY_PHP,
            Self::Css => queries::QUERY_CSS,
        }
    }

    /// Get human-readable name of the language
    pub fn name(&self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::TypeScript => "typescript",
            Self::JavaScript => "javascript",
            Self::Python => "python",
            Self::Go => "go",
            Self::Java => "java",
            Self::C => "c",
            Self::Cpp => "cpp",
            Self::CSharp => "csharp",
            Self::Ruby => "ruby",
            Self::Php => "php",
            Self::Css => "css",
        }
    }

    /// Get all extensions associated with this language
    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            Self::Rust => &["rs"],
            Self::TypeScript => &["ts", "tsx", "mts", "mtsx", "cts"],
            Self::JavaScript => &["js", "jsx", "cjs", "mjs", "mjsx"],
            Self::Python => &["py"],
            Self::Go => &["go"],
            Self::Java => &["java"],
            Self::C => &["c", "h"],
            Self::Cpp => &["cpp", "hpp", "cc", "cxx", "hxx"],
            Self::CSharp => &["cs"],
            Self::Ruby => &["rb"],
            Self::Php => &["php"],
            Self::Css => &["css"],
        }
    }
}

/// Static map from extension to language
static EXTENSION_MAP: OnceLock<HashMap<&'static str, SupportedLanguage>> = OnceLock::new();

/// Get or initialize the extension map
fn get_extension_map() -> &'static HashMap<&'static str, SupportedLanguage> {
    EXTENSION_MAP.get_or_init(|| {
        let languages = [
            SupportedLanguage::Rust,
            SupportedLanguage::TypeScript,
            SupportedLanguage::JavaScript,
            SupportedLanguage::Python,
            SupportedLanguage::Go,
            SupportedLanguage::Java,
            SupportedLanguage::C,
            SupportedLanguage::Cpp,
            SupportedLanguage::CSharp,
            SupportedLanguage::Ruby,
            SupportedLanguage::Php,
            SupportedLanguage::Css,
        ];

        let mut map = HashMap::new();
        for lang in languages {
            for ext in lang.extensions() {
                map.insert(*ext, lang);
            }
        }
        map
    })
}

/// Get language from file extension
pub fn get_language_from_extension(extension: &str) -> Option<SupportedLanguage> {
    get_extension_map().get(extension).copied()
}

/// Get language name from file extension (for backwards compatibility)
pub fn get_language_name_from_extension(extension: &str) -> Option<&'static str> {
    get_language_from_extension(extension).map(|l| l.name())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_language_from_extension() {
        assert_eq!(
            get_language_from_extension("rs"),
            Some(SupportedLanguage::Rust)
        );
        assert_eq!(
            get_language_from_extension("ts"),
            Some(SupportedLanguage::TypeScript)
        );
        assert_eq!(
            get_language_from_extension("tsx"),
            Some(SupportedLanguage::TypeScript)
        );
        assert_eq!(
            get_language_from_extension("js"),
            Some(SupportedLanguage::JavaScript)
        );
        assert_eq!(
            get_language_from_extension("jsx"),
            Some(SupportedLanguage::JavaScript)
        );
        assert_eq!(
            get_language_from_extension("py"),
            Some(SupportedLanguage::Python)
        );
        assert_eq!(
            get_language_from_extension("go"),
            Some(SupportedLanguage::Go)
        );
        assert_eq!(
            get_language_from_extension("java"),
            Some(SupportedLanguage::Java)
        );
        assert_eq!(
            get_language_from_extension("c"),
            Some(SupportedLanguage::C)
        );
        assert_eq!(
            get_language_from_extension("cpp"),
            Some(SupportedLanguage::Cpp)
        );
        assert_eq!(
            get_language_from_extension("cs"),
            Some(SupportedLanguage::CSharp)
        );
        assert_eq!(
            get_language_from_extension("rb"),
            Some(SupportedLanguage::Ruby)
        );
        assert_eq!(
            get_language_from_extension("php"),
            Some(SupportedLanguage::Php)
        );
        assert_eq!(
            get_language_from_extension("css"),
            Some(SupportedLanguage::Css)
        );
        assert_eq!(get_language_from_extension("unknown"), None);
    }

    #[test]
    fn test_language_name() {
        assert_eq!(SupportedLanguage::Rust.name(), "rust");
        assert_eq!(SupportedLanguage::TypeScript.name(), "typescript");
        assert_eq!(SupportedLanguage::CSharp.name(), "csharp");
    }

    #[test]
    fn test_language_extensions() {
        assert!(SupportedLanguage::TypeScript.extensions().contains(&"ts"));
        assert!(SupportedLanguage::TypeScript.extensions().contains(&"tsx"));
        assert!(SupportedLanguage::JavaScript.extensions().contains(&"js"));
        assert!(SupportedLanguage::JavaScript.extensions().contains(&"jsx"));
    }
}
