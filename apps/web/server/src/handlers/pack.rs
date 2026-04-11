//! Pack handler for processing repositories and ZIP files

use axum::{extract::State, http::StatusCode, Json};
use axum::extract::Multipart;
use std::collections::HashSet;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use chrono::Utc;
use tempfile::TempDir;
use uuid::Uuid;

use crate::{error::AppError, state::AppState, types::*};

/// Pack handler - processes remote URLs, uploaded ZIP files, or chunked uploads
pub async fn pack(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<PackResponse>, AppError> {
    // Parse multipart form data
    let mut url: Option<String> = None;
    let mut file_data: Option<Vec<u8>> = None;
    let mut file_name: Option<String> = None;
    let mut upload_id: Option<String> = None;
    let mut format: Option<String> = None;
    let mut options_json: Option<String> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        AppError::bad_request(format!("Failed to read multipart field: {}", e))
    })? {
        let name = field.name().unwrap_or("").to_string();
        
        match name.as_str() {
            "url" => {
                url = Some(field.text().await.map_err(|e| {
                    AppError::bad_request(format!("Failed to read url field: {}", e))
                })?);
            }
            "file" => {
                file_name = field.file_name().map(|s| s.to_string());
                file_data = Some(field.bytes().await.map_err(|e| {
                    AppError::bad_request(format!("Failed to read file data: {}", e))
                })?.to_vec());
            }
            "uploadId" => {
                upload_id = Some(field.text().await.map_err(|e| {
                    AppError::bad_request(format!("Failed to read uploadId: {}", e))
                })?);
            }
            "format" => {
                format = Some(field.text().await.map_err(|e| {
                    AppError::bad_request(format!("Failed to read format: {}", e))
                })?);
            }
            "options" => {
                options_json = Some(field.text().await.map_err(|e| {
                    AppError::bad_request(format!("Failed to read options: {}", e))
                })?);
            }
            _ => {
                // Unknown field, skip it
            }
        }
    }

    // Validate that exactly one source is provided
    let sources_count = [url.is_some(), file_data.is_some(), upload_id.is_some()]
        .iter()
        .filter(|&&x| x)
        .count();

    if sources_count == 0 {
        return Err(AppError::bad_request(
            "Either URL, file, or uploadId must be provided",
        ));
    }

    if sources_count > 1 {
        return Err(AppError::bad_request(
            "Only one of URL, file, or uploadId can be provided",
        ));
    }

    // Parse format (required)
    let format = format.ok_or_else(|| AppError::bad_request("format is required"))?;
    let output_style = match format.as_str() {
        "xml" => repomix::OutputStyle::Xml,
        "markdown" => repomix::OutputStyle::Markdown,
        "plain" => repomix::OutputStyle::Plain,
        _ => return Err(AppError::bad_request(format!("Invalid format: {}", format))),
    };

    // Parse options (optional)
    let pack_options: PackOptions = if let Some(options_str) = options_json {
        serde_json::from_str(&options_str)
            .map_err(|e| AppError::bad_request(format!("Invalid options JSON: {}", e)))?
    } else {
        PackOptions::default()
    };

    // Convert PackOptions to repomix::PackOptions
    let repomix_options = repomix::PackOptions {
        format: Some(output_style),
        compress: pack_options.compress,
        remove_comments: pack_options.remove_comments,
        remove_empty_lines: pack_options.remove_empty_lines,
        show_line_numbers: pack_options.show_line_numbers,
        include_patterns: pack_options.include_patterns.clone(),
        ignore_patterns: pack_options.ignore_patterns.clone(),
        header_text: None,
        instruction_file_path: None,
    };

    let config = repomix::build_config(repomix_options);

    // Process based on source type
    let result = if let Some(repo_url) = url {
        // Remote URL processing
        process_remote_url(&repo_url, config).await?
    } else if let Some(data) = file_data {
        // ZIP file processing
        let fname = file_name.unwrap_or_else(|| "upload.zip".to_string());
        process_zip_file(&fname, &data, config).await?
    } else if let Some(uid) = upload_id {
        // Chunked upload processing
        let uuid = Uuid::parse_str(&uid)
            .map_err(|_| AppError::bad_request("Invalid uploadId format"))?;
        process_chunked_upload(state, uuid, config).await?
    } else {
        unreachable!("Already validated that exactly one source is provided");
    };

    // Build repository name from first file or "unknown"
    let repository = result.file_paths
        .first()
        .and_then(|p| std::path::Path::new(p).components().next())
        .and_then(|c| c.as_os_str().to_str())
        .unwrap_or("unknown")
        .to_string();

    // Clone content to avoid partial move
    let content = result.content.clone();
    let top_files = build_top_files(&result);

    // Build response
    let response = PackResponse {
        content,
        format: format.clone(),
        metadata: PackMetadata {
            repository,
            timestamp: Utc::now().to_rfc3339(),
            summary: Some(PackSummary {
                total_files: result.metrics.total_files,
                total_characters: result.metrics.total_characters,
                total_tokens: result.metrics.total_tokens,
            }),
            top_files: Some(top_files),
        },
    };

    Ok(Json(response))
}

/// Process a remote repository URL
async fn process_remote_url(
    url: &str,
    config: repomix::MergedConfig,
) -> Result<repomix::PackResult, AppError> {
    tracing::info!("Processing remote URL: {}", url);
    
    // Use tokio::task::spawn_blocking for CPU-intensive work
    let url = url.to_string();
    tokio::task::spawn_blocking(move || {
        repomix::pack_remote(&url, None, config)
    })
    .await
    .map_err(|e| AppError::internal(format!("Task join error: {}", e)))?
    .map_err(|e| AppError::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Failed to pack remote repository: {}", e)
    ))
}

/// Process an uploaded ZIP file
async fn process_zip_file(
    file_name: &str,
    data: &[u8],
    config: repomix::MergedConfig,
) -> Result<repomix::PackResult, AppError> {
    tracing::info!("Processing ZIP file: {} ({} bytes)", file_name, data.len());
    
    // Security checks
    const MAX_ZIP_SIZE: usize = 100 * 1024 * 1024; // 100MB
    if data.len() > MAX_ZIP_SIZE {
        return Err(AppError::new(
            StatusCode::PAYLOAD_TOO_LARGE,
            format!("File size exceeds maximum of {}MB", MAX_ZIP_SIZE / 1024 / 1024)
        ));
    }

    // Create temp directory for extraction
    let temp_dir = TempDir::new()
        .map_err(|e| AppError::internal(format!("Failed to create temp directory: {}", e)))?;
    
    let temp_path = temp_dir.path().to_path_buf();
    let data = data.to_vec();

    // Extract and process in blocking task
    tokio::task::spawn_blocking(move || {
        // Extract ZIP file
        extract_zip_secure(&data, &temp_path)?;
        
        // Pack the extracted directory
        repomix::pack_directory(&temp_path, config)
            .map_err(|e| AppError::internal(format!("Failed to pack directory: {}", e)))
    })
    .await
    .map_err(|e| AppError::internal(format!("Task join error: {}", e)))?
}

/// Process a chunked upload by assembling chunks
async fn process_chunked_upload(
    state: Arc<AppState>,
    upload_id: Uuid,
    config: repomix::MergedConfig,
) -> Result<repomix::PackResult, AppError> {
    tracing::info!("Processing chunked upload: {}", upload_id);
    
    // Assemble chunks into a single ZIP file
    let assembled_path = crate::handlers::assemble_chunks(&state, upload_id).await?;
    
    // Read the assembled file
    let data = tokio::fs::read(&assembled_path)
        .await
        .map_err(|e| AppError::internal(format!("Failed to read assembled file: {}", e)))?;
    
    // Get file name from path
    let file_name = assembled_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("upload.zip")
        .to_string();
    
    // Process as ZIP file
    process_zip_file(&file_name, &data, config).await
}

/// Extract ZIP file with security checks
fn extract_zip_secure(data: &[u8], dest: &Path) -> Result<(), AppError> {
    use std::io::Cursor;
    use zip::ZipArchive;

    const MAX_FILES: usize = 50_000;
    const MAX_UNCOMPRESSED_SIZE: u64 = 2 * 1024 * 1024 * 1024; // 2GB
    const MAX_COMPRESSION_RATIO: f64 = 100.0;
    const MAX_PATH_LENGTH: usize = 300;
    const MAX_NESTING_LEVEL: usize = 50;

    let cursor = Cursor::new(data);
    let mut archive = ZipArchive::new(cursor)
        .map_err(|e| AppError::bad_request(format!("Invalid ZIP file: {}", e)))?;

    // Check file count
    if archive.len() > MAX_FILES {
        return Err(AppError::new(
            StatusCode::PAYLOAD_TOO_LARGE,
            format!("ZIP contains too many files: {} > {}", archive.len(), MAX_FILES)
        ));
    }

    // Calculate total uncompressed size and check compression ratio
    let mut total_uncompressed: u64 = 0;
    for i in 0..archive.len() {
        let file = archive.by_index(i)
            .map_err(|e| AppError::bad_request(format!("Failed to read ZIP entry: {}", e)))?;
        total_uncompressed += file.size();
    }

    if total_uncompressed > MAX_UNCOMPRESSED_SIZE {
        return Err(AppError::new(
            StatusCode::PAYLOAD_TOO_LARGE,
            format!("Uncompressed size exceeds maximum: {} > {}", 
                total_uncompressed, MAX_UNCOMPRESSED_SIZE)
        ));
    }

    let compression_ratio = total_uncompressed as f64 / data.len() as f64;
    if compression_ratio > MAX_COMPRESSION_RATIO {
        return Err(AppError::bad_request(format!(
            "Suspicious compression ratio: {:.2} > {}", 
            compression_ratio, MAX_COMPRESSION_RATIO
        )));
    }

    // Extract files
    std::fs::create_dir_all(dest)
        .map_err(|e| AppError::internal(format!("Failed to create destination: {}", e)))?;

    let mut processed_paths = HashSet::new();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| AppError::bad_request(format!("Failed to read ZIP entry: {}", e)))?;
        
        let file_path = file.name().to_string();
        
        // Skip directories
        if file.is_dir() {
            continue;
        }

        if file.is_symlink() {
            return Err(AppError::bad_request(format!(
                "Symbolic links are not allowed in ZIP archives: {}",
                file_path
            )));
        }

        if !file.is_file() {
            return Err(AppError::bad_request(format!(
                "Unsupported ZIP entry type: {}",
                file_path
            )));
        }

        // Security checks
        if file_path.len() > MAX_PATH_LENGTH {
            return Err(AppError::bad_request(format!(
                "File path too long: {} > {}", file_path.len(), MAX_PATH_LENGTH
            )));
        }

        let nesting_level = file_path.matches('/').count();
        if nesting_level > MAX_NESTING_LEVEL {
            return Err(AppError::bad_request(format!(
                "Path nesting too deep: {} > {}", nesting_level, MAX_NESTING_LEVEL
            )));
        }

        // Prevent path traversal and duplicate normalized targets
        let enclosed_path = file.enclosed_name().ok_or_else(|| {
            AppError::bad_request(format!("Path traversal detected: {}", file_path))
        })?;
        let normalized_path = normalize_enclosed_path(&enclosed_path);

        if normalized_path.as_os_str().is_empty() {
            return Err(AppError::bad_request(format!(
                "Invalid ZIP entry path: {}",
                file_path
            )));
        }

        let out_path = dest.join(&normalized_path);
        
        if !out_path.starts_with(dest) {
            return Err(AppError::bad_request(format!(
                "Path traversal detected: {}", file_path
            )));
        }

        if !processed_paths.insert(out_path.clone()) {
            return Err(AppError::bad_request(format!(
                "Duplicate file path detected: {}. This could indicate a malicious archive.",
                file_path
            )));
        }

        // Create parent directories
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| AppError::internal(format!("Failed to create directory: {}", e)))?;
        }

        // Extract file
        let mut out_file = std::fs::File::create(&out_path)
            .map_err(|e| AppError::internal(format!("Failed to create file: {}", e)))?;
        
        std::io::copy(&mut file, &mut out_file)
            .map_err(|e| AppError::internal(format!("Failed to extract file: {}", e)))?;
    }

    Ok(())
}

fn normalize_enclosed_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::Normal(part) => normalized.push(part),
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Prefix(_) | Component::RootDir => {}
        }
    }

    normalized
}

/// Build top files list from pack result
fn build_top_files(_result: &repomix::PackResult) -> Vec<TopFile> {
    // Use file_char_counts from metrics to build top files
    _result.metrics.file_char_counts
        .iter()
        .take(10)
        .map(|fm| TopFile {
            path: fm.path.clone(),
            char_count: fm.characters,
            token_count: fm.tokens,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::extract_zip_secure;
    use axum::http::StatusCode;
    use std::io::{Cursor, Write};
    use tempfile::tempdir;
    use zip::{write::SimpleFileOptions, ZipWriter};

    fn build_zip(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let cursor = Cursor::new(Vec::new());
        let mut writer = ZipWriter::new(cursor);
        let options = SimpleFileOptions::default();

        for (path, contents) in entries {
            writer.start_file(path, options).unwrap();
            writer.write_all(contents).unwrap();
        }

        writer.finish().unwrap().into_inner()
    }

    #[test]
    fn rejects_parent_dir_escape_paths() {
        let archive = build_zip(&[("../repomix-1234-evil/pwn.txt", b"owned")]);
        let dest = tempdir().unwrap();

        let err = extract_zip_secure(&archive, dest.path()).unwrap_err();

        assert_eq!(err.status, StatusCode::BAD_REQUEST);
        assert!(err.message.contains("Path traversal detected"));
        assert!(!dest.path().join("../repomix-1234-evil/pwn.txt").exists());
    }

    #[test]
    fn rejects_absolute_paths() {
        let archive = build_zip(&[("/tmp/pwn.txt", b"owned")]);
        let dest = tempdir().unwrap();

        let err = extract_zip_secure(&archive, dest.path()).unwrap_err();

        assert_eq!(err.status, StatusCode::BAD_REQUEST);
        assert!(err.message.contains("Path traversal detected"));
    }

    #[test]
    fn rejects_duplicate_normalized_paths() {
        let archive = build_zip(&[("safe.txt", b"one"), ("nested/../safe.txt", b"two")]);
        let dest = tempdir().unwrap();

        let err = extract_zip_secure(&archive, dest.path()).unwrap_err();

        assert_eq!(err.status, StatusCode::BAD_REQUEST);
        assert!(err.message.contains("Duplicate file path detected"));
    }

    #[test]
    fn extracts_safe_paths_after_normalization() {
        let archive = build_zip(&[("nested/../safe.txt", b"ok")]);
        let dest = tempdir().unwrap();

        extract_zip_secure(&archive, dest.path()).unwrap();

        assert_eq!(std::fs::read(dest.path().join("safe.txt")).unwrap(), b"ok");
    }
}
