//! File collection and reading
//!
//! Provides parallel file reading with encoding detection and binary file filtering.

use anyhow::{Context, Result};
use content_inspector::{inspect, ContentType};
use encoding_rs::Encoding;
use rayon::prelude::*;
use std::fs;
use std::path::Path;

/// Reason why a file was skipped
#[derive(Debug, Clone, PartialEq)]
pub enum FileSkipReason {
    /// File has a binary extension
    BinaryExtension,
    /// File content is binary
    BinaryContent,
    /// File exceeds size limit
    SizeLimit,
    /// File has encoding errors
    EncodingError,
    /// Failed to read file
    ReadError,
}

impl std::fmt::Display for FileSkipReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BinaryExtension => write!(f, "Binary file (by extension)"),
            Self::BinaryContent => write!(f, "Binary file (by content)"),
            Self::SizeLimit => write!(f, "File exceeds size limit"),
            Self::EncodingError => write!(f, "Encoding error"),
            Self::ReadError => write!(f, "Failed to read file"),
        }
    }
}

/// Information about a skipped file
#[derive(Debug, Clone)]
pub struct SkippedFile {
    /// Path of the skipped file
    pub path: String,
    /// Reason why the file was skipped
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
    "png", "jpg", "jpeg", "gif", "bmp", "ico", "webp", "svg", "tiff", "tif", "psd", "raw",
    // Audio
    "mp3", "wav", "ogg", "flac", "aac", "wma", "m4a", "mid", "midi",
    // Video
    "mp4", "avi", "mkv", "mov", "wmv", "flv", "webm", "m4v", "mpeg", "mpg",
    // Archives
    "zip", "tar", "gz", "bz2", "xz", "7z", "rar", "iso", "dmg", "pkg", "deb", "rpm",
    // Documents
    "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "odt", "ods", "odp",
    // Executables
    "exe", "dll", "so", "dylib", "bin", "o", "a", "lib", "msi",
    // Fonts
    "ttf", "otf", "woff", "woff2", "eot",
    // Database
    "db", "sqlite", "sqlite3", "mdb",
    // Others
    "pyc", "pyo", "class", "jar", "war", "ear",
    "wasm",
];

/// Check if a file has a binary extension
fn is_binary_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| BINARY_EXTENSIONS.contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
}

/// Check if content is binary
fn is_binary_content(content: &[u8]) -> bool {
    match inspect(content) {
        ContentType::BINARY => true,
        ContentType::UTF_8
        | ContentType::UTF_8_BOM
        | ContentType::UTF_16LE
        | ContentType::UTF_16BE
        | ContentType::UTF_32LE
        | ContentType::UTF_32BE => false,
    }
}

/// Detect encoding and decode content
fn decode_content(content: &[u8]) -> Result<String, FileSkipReason> {
    // Check if content is binary
    if is_binary_content(content) {
        return Err(FileSkipReason::BinaryContent);
    }
    
    // Try to detect encoding
    let (encoding, confidence) = detect_encoding(content);
    
    // Decode with detected encoding
    let (decoded, _, had_errors) = encoding.decode(content);
    
    if had_errors && confidence < 0.8 {
        // If there are errors and low confidence, try UTF-8 with strict check
        match std::str::from_utf8(content) {
            Ok(s) => {
                // Strip BOM if present
                let s = s.strip_prefix('\u{FEFF}').unwrap_or(s);
                return Ok(s.to_string());
            }
            Err(_) => return Err(FileSkipReason::EncodingError),
        }
    }
    
    // Strip BOM if present
    let result = decoded.strip_prefix('\u{FEFF}').unwrap_or(&decoded);
    
    // Check for replacement characters indicating decoding errors
    if result.contains('\u{FFFD}') {
        // For UTF-8, try strict decoding
        if encoding == encoding_rs::UTF_8 {
            match std::str::from_utf8(content) {
                Ok(s) => {
                    let s = s.strip_prefix('\u{FEFF}').unwrap_or(s);
                    return Ok(s.to_string());
                }
                Err(_) => return Err(FileSkipReason::EncodingError),
            }
        }
        return Err(FileSkipReason::EncodingError);
    }
    
    Ok(result.to_string())
}

/// Detect encoding of content
fn detect_encoding(content: &[u8]) -> (&'static Encoding, f32) {
    // Check for BOM first
    if content.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return (encoding_rs::UTF_8, 1.0);
    }
    if content.starts_with(&[0xFF, 0xFE]) {
        return (encoding_rs::UTF_16LE, 1.0);
    }
    if content.starts_with(&[0xFE, 0xFF]) {
        return (encoding_rs::UTF_16BE, 1.0);
    }
    
    // Default to UTF-8
    (encoding_rs::UTF_8, 0.95)
}

/// Read a single file
fn read_file(root_dir: &Path, rel_path: &str, max_file_size: usize) -> Result<CollectedFile, SkippedFile> {
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
    let results: Vec<Result<CollectedFile, SkippedFile>> = file_paths
        .par_iter()
        .map(|rel_path| read_file(root_dir, rel_path, max_file_size))
        .collect();
    
    let mut files = Vec::new();
    let mut skipped = Vec::new();
    
    for result in results {
        match result {
            Ok(file) => files.push(file),
            Err(skip) => skipped.push(skip),
        }
    }
    
    Ok(CollectResult { files, skipped })
}

/// Collect files with a progress callback (sequential for progress tracking)
pub fn collect_files_with_progress<F>(
    root_dir: &Path,
    file_paths: &[String],
    max_file_size: usize,
    progress_callback: F,
) -> Result<CollectResult>
where
    F: Fn(usize, usize, &str),
{
    let total = file_paths.len();
    let mut files = Vec::new();
    let mut skipped = Vec::new();
    
    for (i, rel_path) in file_paths.iter().enumerate() {
        progress_callback(i + 1, total, rel_path);
        
        match read_file(root_dir, rel_path, max_file_size) {
            Ok(file) => files.push(file),
            Err(skip) => skipped.push(skip),
        }
    }
    
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
    fn test_is_binary_content() {
        // Text content
        assert!(!is_binary_content(b"Hello, world!"));
        assert!(!is_binary_content("fn main() {}".as_bytes()));
        
        // Binary content (contains null bytes)
        assert!(is_binary_content(&[0x00, 0x01, 0x02, 0x03]));
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
        
        fs::write(root.join("image.png"), &[0x89, 0x50, 0x4E, 0x47])?;
        fs::write(root.join("code.rs"), "fn main() {}")?;
        
        let paths = vec!["image.png".to_string(), "code.rs".to_string()];
        let result = collect_files(root, &paths, 50 * 1024 * 1024)?;
        
        assert_eq!(result.files.len(), 1);
        assert_eq!(result.skipped.len(), 1);
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
        assert_eq!(result.skipped[0].reason, FileSkipReason::SizeLimit);
        
        Ok(())
    }
}
