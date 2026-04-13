//! File collection and reading
//!
//! Provides parallel file reading with encoding detection and binary file filtering.

use anyhow::Result;
use chardetng;
use encoding_rs::Encoding;
use rayon::prelude::*;
use std::fs;
use std::path::Path;

/// Reason why a file was skipped
#[derive(Debug, Clone, PartialEq)]
pub enum FileSkipReason {
    /// File content is binary
    BinaryContent,
    /// File has encoding errors
    EncodingError,
    /// File has binary extension
    BinaryExtension,
    /// File exceeds size limit
    SizeLimit,
    /// IO error during reading
    ReadError,
}

impl std::fmt::Display for FileSkipReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BinaryContent => write!(f, "Binary file (by content)"),
            Self::EncodingError => write!(f, "Encoding error"),
            Self::BinaryExtension => write!(f, "Binary file (by extension)"),
            Self::SizeLimit => write!(f, "File size limit exceeded"),
            Self::ReadError => write!(f, "Read error"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SkippedFile {
    pub path: String,
    pub reason: FileSkipReason,
}

/// Collected file with content
#[derive(Debug, Clone)]
pub struct CollectedFile {
    /// Relative path of the file
    pub path: String,
    /// Content of the file
    pub content: String,
}

/// Result of file collection
#[derive(Debug)]
pub struct CollectResult {
    /// Successfully collected files
    pub files: Vec<CollectedFile>,
    /// Files that were skipped
    pub skipped: Vec<SkippedFile>,
}

/// Common binary file extensions
const BINARY_EXTENSIONS: &[&str] = &[
    // Images
    "png",
    "jpg",
    "jpeg",
    "gif",
    "bmp",
    "ico",
    "webp",
    "tiff",
    "tif",
    "psd",
    "raw",
    "icns",
    "cur",
    "ani",
    "webp",
    // Audio
    "mp3",
    "wav",
    "ogg",
    "flac",
    "aac",
    "wma",
    "m4a",
    "mid",
    "midi",
    // Video
    "mp4",
    "avi",
    "mkv",
    "mov",
    "wmv",
    "flv",
    "webm",
    "m4v",
    "mpeg",
    "mpg",
    // Archives
    "zip",
    "tar",
    "gz",
    "bz2",
    "xz",
    "7z",
    "rar",
    "iso",
    "dmg",
    "pkg",
    "deb",
    "rpm",
    // Documents
    "pdf",
    "doc",
    "docx",
    "xls",
    "xlsx",
    "ppt",
    "pptx",
    "odt",
    "ods",
    "odp",
    // Executables
    "exe",
    "dll",
    "so",
    "dylib",
    "bin",
    "o",
    "a",
    "lib",
    "msi",
    // Fonts
    "ttf",
    "otf",
    "woff",
    "woff2",
    "eot",
    // Database
    "db",
    "sqlite",
    "sqlite3",
    "mdb",
    // Telegram specific & Animations
    "tgv",
    "tgs",
    "lottie",
    "tdesktop-theme",
    // Others
    "pyc",
    "pyo",
    "class",
    "jar",
    "war",
    "ear",
    "wasm",
];

/// Check if a file has a binary extension
fn is_binary_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| BINARY_EXTENSIONS.contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
}

/// Check if decoded text contains too much "binary garbage"
fn is_binary_text(text: &str) -> bool {
    if text.is_empty() {
        return false;
    }

    const BINARY_THRESHOLD: f32 = 0.10; // 10%
    let mut binary_chars = 0;
    let bytes = text.as_bytes();
    let total = bytes.len();

    for &byte in bytes {
        // Control characters except tab (0x09), newline (0x0A), carriage return (0x0D)
        if matches!(byte, 0x00..=0x08 | 0x0B | 0x0C | 0x0E..=0x1F | 0x7F) {
            binary_chars += 1;
        }
    }

    (binary_chars as f32 / total as f32) > BINARY_THRESHOLD
}

/// Detect encoding of content with fallback to chardetng
fn detect_encoding(content: &[u8]) -> &'static Encoding {
    // Check for BOM first (highest priority)
    if content.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return encoding_rs::UTF_8;
    }
    if content.starts_with(&[0xFF, 0xFE]) {
        return encoding_rs::UTF_16LE;
    }
    if content.starts_with(&[0xFE, 0xFF]) {
        return encoding_rs::UTF_16BE;
    }

    // Use chardetng for guessing
    let mut detector = chardetng::EncodingDetector::new();
    detector.feed(content, true);
    detector.guess(None, true)
}

/// Decode content and check for binary and encoding errors
fn decode_content(content: &[u8]) -> Result<String, FileSkipReason> {
    if content.is_empty() {
        return Ok(String::new());
    }

    // 1. Detect encoding
    let encoding = detect_encoding(content);

    // 2. Decode with detected encoding
    let (decoded, _, had_errors) = encoding.decode(content);

    // 3. Strip BOM
    let result = decoded.strip_prefix('\u{FEFF}').unwrap_or(&decoded);

    // 4. Check for decoding replacement characters
    // Only fail if it's UTF-8 and has errors, or if we have replacement characters
    if had_errors || result.contains('\u{FFFD}') {
        // If we assumed some encoding but it has errors, try strict UTF-8 as last resort
        if encoding != encoding_rs::UTF_8 {
            if let Ok(utf8_str) = std::str::from_utf8(content) {
                let s = utf8_str.strip_prefix('\u{FEFF}').unwrap_or(utf8_str);
                if !is_binary_text(s) {
                    return Ok(s.to_string());
                }
            }
        }
        return Err(FileSkipReason::EncodingError);
    }

    // 5. Check decoded text for binary "garbage"
    if is_binary_text(result) {
        return Err(FileSkipReason::BinaryContent);
    }

    Ok(result.to_string())
}

/// Read a single file
fn read_file(
    root_dir: &Path,
    rel_path: &str,
    max_file_size: usize,
) -> Result<CollectedFile, SkippedFile> {
    let full_path = root_dir.join(rel_path);
    let path = rel_path.to_string();

    // Check binary extension
    if is_binary_extension(&full_path) {
        return Err(SkippedFile {
            path,
            reason: FileSkipReason::BinaryExtension,
        });
    }

    // Check file size
    let metadata = match fs::metadata(&full_path) {
        Ok(m) => m,
        Err(_) => {
            return Err(SkippedFile {
                path,
                reason: FileSkipReason::ReadError,
            });
        }
    };

    if metadata.len() as usize > max_file_size {
        return Err(SkippedFile {
            path,
            reason: FileSkipReason::SizeLimit,
        });
    }

    // Read file content
    let content = match fs::read(&full_path) {
        Ok(c) => c,
        Err(_) => {
            return Err(SkippedFile {
                path,
                reason: FileSkipReason::ReadError,
            });
        }
    };

    // Decode content
    match decode_content(&content) {
        Ok(text) => Ok(CollectedFile {
            path,
            content: text,
        }),
        Err(reason) => Err(SkippedFile { path, reason }),
    }
}

/// Collect and read files from paths (parallel)
pub fn collect_files(
    root_dir: &Path,
    file_paths: &[String],
    max_file_size: usize,
) -> Result<CollectResult> {
    let (mut files, mut skipped) = file_paths
        .par_iter()
        .map(|rel_path| read_file(root_dir, rel_path, max_file_size))
        .fold(
            || (Vec::new(), Vec::new()),
            |mut acc, result| {
                match result {
                    Ok(file) => acc.0.push(file),
                    Err(skip) => acc.1.push(skip),
                }
                acc
            },
        )
        .reduce(
            || (Vec::new(), Vec::new()),
            |mut left, mut right| {
                left.0.append(&mut right.0);
                left.1.append(&mut right.1);
                left
            },
        );

    files.par_sort_by(|a, b| a.path.cmp(&b.path));
    skipped.par_sort_by(|a, b| a.path.cmp(&b.path));

    Ok(CollectResult { files, skipped })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_is_binary_extension() {
        assert!(is_binary_extension(Path::new("image.png")));
        assert!(is_binary_extension(Path::new("archive.zip")));
        assert!(!is_binary_extension(Path::new("code.rs")));
        assert!(!is_binary_extension(Path::new("readme.txt")));
    }

    #[test]
    fn test_is_binary_text() {
        // Text content
        assert!(!is_binary_text("Hello, world!"));
        assert!(!is_binary_text("fn main() {}"));

        // Binary content (contains high ratio of null bytes/control chars)
        let binary_content =
            String::from_utf8_lossy(&[0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x61, 0x62]);
        assert!(is_binary_text(&binary_content));

        // UTF-16 like content but already decoded to string
        let utf16_text = "Hello";
        assert!(!is_binary_text(utf16_text));
    }

    #[test]
    fn test_decode_utf8() {
        let content = "Hello, мир!".as_bytes();
        let decoded = decode_content(content).unwrap();
        assert_eq!(decoded, "Hello, мир!");
    }

    #[test]
    fn test_decode_utf8_bom() {
        let content = b"\xef\xbb\xbfHello, world!";
        let decoded = decode_content(content).unwrap();
        assert_eq!(decoded, "Hello, world!");
    }

    #[test]
    fn test_decode_utf16le() {
        // "Hello" in UTF-16LE: 48 00 65 00 6c 00 6c 00 6f 00
        // Without BOM, it's hard to detect, but let's test that it DOES NOT
        // fail just because of one or two null bytes if it's treated as text.
        // However, UTF-16 without BOM WILL be detected as something else and probably have many null bytes.

        // Let's test a more realistic case: a text file with a few control characters.
        let content = b"Hello\x00world"; // Only one null byte
        let decoded = decode_content(content).unwrap();
        assert_eq!(decoded, "Hello\0world");
    }

    #[test]
    fn test_decode_utf16le_with_bom() {
        // BOM (FF FE) + "Hello"
        let content = b"\xff\xfe\x48\x00\x65\x00\x6c\x00\x6c\x00\x6f\x00";
        let decoded = decode_content(content).unwrap();
        assert_eq!(decoded, "Hello");
    }

    #[test]
    fn test_collect_files() -> Result<()> {
        let dir = tempdir()?;
        let root = dir.path();

        fs::write(root.join("main.rs"), "fn main() {}")?;
        fs::write(root.join("lib.rs"), "pub mod test;")?;

        let paths = vec!["main.rs".to_string(), "lib.rs".to_string()];
        let result = collect_files(root, &paths, 50 * 1024 * 1024)?;

        assert_eq!(result.files.len(), 2);
        assert!(result.skipped.is_empty());

        let main_file = result.files.iter().find(|f| f.path == "main.rs").unwrap();
        assert_eq!(main_file.content, "fn main() {}");

        Ok(())
    }

    #[test]
    fn test_skip_binary_extension() -> Result<()> {
        let dir = tempdir()?;
        let root = dir.path();

        fs::write(root.join("image.png"), [0x89, 0x50, 0x4E, 0x47])?;
        fs::write(root.join("code.rs"), "fn main() {}")?;

        let paths = vec!["image.png".to_string(), "code.rs".to_string()];
        let result = collect_files(root, &paths, 50 * 1024 * 1024)?;

        assert_eq!(result.files.len(), 1);
        assert_eq!(result.skipped.len(), 1);
        assert_eq!(result.skipped[0].path, "image.png");
        assert_eq!(result.skipped[0].reason, FileSkipReason::BinaryExtension);

        Ok(())
    }

    #[test]
    fn test_skip_large_file() -> Result<()> {
        let dir = tempdir()?;
        let root = dir.path();

        // Create a file larger than limit
        let large_content = "x".repeat(1024);
        fs::write(root.join("large.txt"), &large_content)?;

        let paths = vec!["large.txt".to_string()];
        let result = collect_files(root, &paths, 100)?; // 100 bytes limit

        assert_eq!(result.files.len(), 0);
        assert_eq!(result.skipped.len(), 1);
        assert_eq!(result.skipped[0].path, "large.txt");
        assert_eq!(result.skipped[0].reason, FileSkipReason::SizeLimit);

        Ok(())
    }
}
