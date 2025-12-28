//! Language detection, mapping, and tree-sitter parser initialization

use std::collections::HashMap;
use std::sync::OnceLock;
use arborium_tree_sitter::Language;

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
    Vue,
}

impl SupportedLanguage {
    /// Get the tree-sitter Language for this language
    pub fn get_ts_language(&self) -> Language {
        match self {
            Self::Rust => arborium_rust::language().into(),
            Self::TypeScript => arborium_typescript::language().into(),
            Self::JavaScript => arborium_javascript::language().into(),
            Self::Python => arborium_python::language().into(),
            Self::Go => arborium_go::language().into(),
            Self::Java => arborium_java::language().into(),
            Self::C => arborium_c::language().into(),
            Self::Cpp => arborium_cpp::language().into(),
            Self::CSharp => arborium_c_sharp::language().into(),
            Self::Ruby => arborium_ruby::language().into(),
            Self::Php => arborium_php::language().into(),
            Self::Css => arborium_css::language().into(),
            Self::Vue => arborium_vue::language().into(),
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
            Self::Vue => queries::QUERY_VUE,
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
            Self::Vue => "vue",
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
            Self::Vue => &["vue"],
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
            SupportedLanguage::Vue,
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
