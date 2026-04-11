//! Pack handler for processing repositories and uploaded archives

use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use serde::Deserialize;
use std::collections::HashSet;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use tempfile::TempDir;
use uuid::Uuid;

use crate::{error::AppError, state::AppState, types::*};

const MAX_DIRECT_UPLOAD_SIZE: usize = 100 * 1024 * 1024;
const MAX_ZIP_SIZE: usize = 100 * 1024 * 1024;
const MAX_FILES: usize = 50_000;
const MAX_UNCOMPRESSED_SIZE: u64 = 2 * 1024 * 1024 * 1024;
const MAX_COMPRESSION_RATIO: f64 = 100.0;
const MAX_PATH_LENGTH: usize = 300;
const MAX_NESTING_LEVEL: usize = 50;
const MAX_BROWSER_FILE_SELECTION_FILES: usize = 2_000;

#[derive(Debug, Clone)]
pub(crate) struct PackJob {
    pub format: String,
    pub options: PackOptions,
    pub source: PackSource,
}

#[derive(Debug, Clone)]
pub(crate) enum PackSource {
    Url(String),
    ZipUpload { file_name: String, data: Vec<u8> },
    ChunkedUpload { upload_id: Uuid },
    FolderUpload { files: Vec<FolderUploadFile> },
}

#[derive(Debug, Clone)]
pub(crate) struct FolderUploadFile {
    pub relative_path: String,
    pub data: Vec<u8>,
}

#[derive(Debug, Deserialize)]
struct FolderUploadManifest {
    paths: Vec<String>,
}

/// Pack handler - processes remote URLs, uploaded ZIP files, chunked uploads, or direct folder uploads
pub async fn pack(
    State(state): State<Arc<AppState>>,
    multipart: Multipart,
) -> Result<Json<PackResponse>, AppError> {
    let job = parse_api_pack_job(multipart).await?;
    let response = execute_pack_job(state, job).await?;
    Ok(Json(response))
}

pub(crate) async fn execute_pack_job(
    state: Arc<AppState>,
    job: PackJob,
) -> Result<PackResponse, AppError> {
    let output_style = parse_output_style(&job.format)?;

    let repomix_options = repomix::PackOptions {
        format: Some(output_style),
        compress: job.options.compress,
        remove_comments: job.options.remove_comments,
        remove_empty_lines: job.options.remove_empty_lines,
        show_line_numbers: job.options.show_line_numbers,
        file_summary: job.options.file_summary,
        directory_structure: job.options.directory_structure,
        include_patterns: job.options.include_patterns.clone(),
        ignore_patterns: job.options.ignore_patterns.clone(),
        output_parsable: job.options.output_parsable,
        header_text: None,
        instruction_file_path: None,
    };

    let config = repomix::build_config(repomix_options);

    let (repository_hint, result) = match job.source {
        PackSource::Url(url) => (
            Some(repository_hint_from_remote_url(&url)),
            process_remote_url(&url, config).await?,
        ),
        PackSource::ZipUpload { file_name, data } => (
            Some(file_stem_or_default(&file_name, "upload")),
            process_zip_file(&file_name, &data, config).await?,
        ),
        PackSource::ChunkedUpload { upload_id } => (
            Some(chunked_upload_repository_hint(&state, upload_id).await?),
            process_chunked_upload(state, upload_id, config).await?,
        ),
        PackSource::FolderUpload { files } => (
            repository_hint_from_folder(&files),
            process_folder_upload(files, config).await?,
        ),
    };

    Ok(build_pack_response(result, job.format, repository_hint))
}

async fn parse_api_pack_job(mut multipart: Multipart) -> Result<PackJob, AppError> {
    let mut url: Option<String> = None;
    let mut zip_upload: Option<(String, Vec<u8>)> = None;
    let mut upload_id: Option<String> = None;
    let mut format: Option<String> = None;
    let mut options_json: Option<String> = None;
    let mut folder_manifest: Option<String> = None;
    let mut folder_files: Vec<Vec<u8>> = Vec::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::bad_request(format!("Failed to read multipart field: {e}")))?
    {
        let name = field.name().unwrap_or("").to_string();

        match name.as_str() {
            "url" => {
                url = Some(field.text().await.map_err(|e| {
                    AppError::bad_request(format!("Failed to read url field: {e}"))
                })?);
            }
            "file" => {
                if zip_upload.is_some() {
                    return Err(AppError::bad_request("Only one ZIP file can be uploaded"));
                }

                let file_name = field.file_name().unwrap_or("upload.zip").to_string();
                let data = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::bad_request(format!("Failed to read file data: {e}")))?
                    .to_vec();

                zip_upload = Some((file_name, data));
            }
            "uploadId" => {
                upload_id =
                    Some(field.text().await.map_err(|e| {
                        AppError::bad_request(format!("Failed to read uploadId: {e}"))
                    })?);
            }
            "format" => {
                format =
                    Some(field.text().await.map_err(|e| {
                        AppError::bad_request(format!("Failed to read format: {e}"))
                    })?);
            }
            "options" => {
                options_json =
                    Some(field.text().await.map_err(|e| {
                        AppError::bad_request(format!("Failed to read options: {e}"))
                    })?);
            }
            "folderManifest" => {
                folder_manifest = Some(field.text().await.map_err(|e| {
                    AppError::bad_request(format!("Failed to read folder manifest: {e}"))
                })?);
            }
            "folderFiles" => {
                let bytes = field.bytes().await.map_err(|e| {
                    AppError::bad_request(format!("Failed to read folder file data: {e}"))
                })?;
                folder_files.push(bytes.to_vec());
            }
            _ => {}
        }
    }

    let format = format.ok_or_else(|| AppError::bad_request("format is required"))?;
    let options = if let Some(options_str) = options_json {
        serde_json::from_str(&options_str)
            .map_err(|e| AppError::bad_request(format!("Invalid options JSON: {e}")))?
    } else {
        PackOptions::default()
    };

    let has_folder = folder_manifest.is_some() || !folder_files.is_empty();
    let sources_count = [
        url.is_some(),
        zip_upload.is_some(),
        upload_id.is_some(),
        has_folder,
    ]
    .iter()
    .filter(|&&present| present)
    .count();

    if sources_count == 0 {
        return Err(AppError::bad_request(
            "Either url, file, uploadId, or folderFiles must be provided",
        ));
    }

    if sources_count > 1 {
        return Err(AppError::bad_request(
            "Only one of url, file, uploadId, or folderFiles can be provided",
        ));
    }

    let source = if let Some(repo_url) = url {
        PackSource::Url(repo_url)
    } else if let Some((file_name, data)) = zip_upload {
        PackSource::ZipUpload { file_name, data }
    } else if let Some(raw_upload_id) = upload_id {
        let upload_id = Uuid::parse_str(&raw_upload_id)
            .map_err(|_| AppError::bad_request("Invalid uploadId format"))?;
        PackSource::ChunkedUpload { upload_id }
    } else {
        PackSource::FolderUpload {
            files: build_folder_upload(folder_manifest, folder_files)?,
        }
    };

    Ok(PackJob {
        format,
        options,
        source,
    })
}

pub(crate) fn build_folder_upload(
    manifest: Option<String>,
    file_payloads: Vec<Vec<u8>>,
) -> Result<Vec<FolderUploadFile>, AppError> {
    let manifest = manifest.ok_or_else(|| {
        AppError::bad_request("folderManifest is required when uploading folderFiles")
    })?;

    if file_payloads.is_empty() {
        return Err(AppError::bad_request(
            "At least one folder file must be provided",
        ));
    }

    let paths = parse_folder_manifest(&manifest)?;

    if paths.len() != file_payloads.len() {
        return Err(AppError::bad_request(format!(
            "folderManifest/file count mismatch: {} paths for {} files",
            paths.len(),
            file_payloads.len()
        )));
    }

    if paths.len() > MAX_FILES {
        return Err(AppError::new(
            StatusCode::PAYLOAD_TOO_LARGE,
            format!(
                "Folder contains too many files: {} > {}",
                paths.len(),
                MAX_FILES
            ),
        ));
    }

    let total_size: usize = file_payloads.iter().map(Vec::len).sum();
    if total_size > MAX_DIRECT_UPLOAD_SIZE {
        return Err(AppError::new(
            StatusCode::PAYLOAD_TOO_LARGE,
            format!(
                "Folder upload exceeds maximum of {}MB",
                MAX_DIRECT_UPLOAD_SIZE / 1024 / 1024
            ),
        ));
    }

    Ok(paths
        .into_iter()
        .zip(file_payloads)
        .map(|(relative_path, data)| FolderUploadFile {
            relative_path,
            data,
        })
        .collect())
}

fn parse_folder_manifest(manifest: &str) -> Result<Vec<String>, AppError> {
    if let Ok(parsed) = serde_json::from_str::<FolderUploadManifest>(manifest) {
        return Ok(parsed.paths);
    }

    serde_json::from_str::<Vec<String>>(manifest)
        .map_err(|e| AppError::bad_request(format!("Invalid folder manifest JSON: {e}")))
}

fn parse_output_style(format: &str) -> Result<repomix::OutputStyle, AppError> {
    match format {
        "xml" => Ok(repomix::OutputStyle::Xml),
        "markdown" => Ok(repomix::OutputStyle::Markdown),
        "plain" => Ok(repomix::OutputStyle::Plain),
        _ => Err(AppError::bad_request(format!("Invalid format: {format}"))),
    }
}

fn build_pack_response(
    result: repomix::PackResult,
    format: String,
    repository_hint: Option<String>,
) -> PackResponse {
    let repository = repository_hint.unwrap_or_else(|| {
        result
            .file_paths
            .first()
            .and_then(|p| std::path::Path::new(p).components().next())
            .and_then(|c| c.as_os_str().to_str())
            .unwrap_or("unknown")
            .to_string()
    });

    let content = result.content.clone();
    let top_files = build_top_files(&result);
    let all_files = build_all_files(&result);

    PackResponse {
        content,
        format,
        metadata: PackMetadata {
            repository,
            timestamp: Utc::now().to_rfc3339(),
            summary: Some(PackSummary {
                total_files: result.metrics.total_files,
                total_characters: result.metrics.total_characters,
                total_tokens: result.metrics.total_tokens,
            }),
            top_files: Some(top_files),
            all_files,
        },
    }
}

fn repository_hint_from_remote_url(url: &str) -> String {
    url.trim_end_matches('/')
        .split('/')
        .filter(|segment| !segment.is_empty())
        .next_back()
        .unwrap_or("remote-repository")
        .trim_end_matches(".git")
        .to_string()
}

fn repository_hint_from_folder(files: &[FolderUploadFile]) -> Option<String> {
    files.first().and_then(|file| {
        Path::new(&file.relative_path)
            .components()
            .next()
            .and_then(|component| component.as_os_str().to_str())
            .map(str::to_string)
    })
}

fn file_stem_or_default(file_name: &str, fallback: &str) -> String {
    Path::new(file_name)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .filter(|stem| !stem.is_empty())
        .unwrap_or(fallback)
        .to_string()
}

async fn chunked_upload_repository_hint(
    state: &AppState,
    upload_id: Uuid,
) -> Result<String, AppError> {
    let uploads = state.uploads.read().await;
    let session = uploads
        .get(&upload_id)
        .ok_or_else(|| AppError::not_found("Upload session not found or expired"))?;

    Ok(file_stem_or_default(&session.file_name, "upload"))
}

/// Process a remote repository URL
async fn process_remote_url(
    url: &str,
    config: repomix::MergedConfig,
) -> Result<repomix::PackResult, AppError> {
    tracing::info!("Processing remote URL: {}", url);

    let url = url.to_string();
    tokio::task::spawn_blocking(move || repomix::pack_remote(&url, None, config))
        .await
        .map_err(|e| AppError::internal(format!("Task join error: {e}")))?
        .map_err(|e| {
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to pack remote repository: {e}"),
            )
        })
}

/// Process an uploaded ZIP file
async fn process_zip_file(
    file_name: &str,
    data: &[u8],
    config: repomix::MergedConfig,
) -> Result<repomix::PackResult, AppError> {
    tracing::info!("Processing ZIP file: {} ({} bytes)", file_name, data.len());

    if data.len() > MAX_ZIP_SIZE {
        return Err(AppError::new(
            StatusCode::PAYLOAD_TOO_LARGE,
            format!(
                "File size exceeds maximum of {}MB",
                MAX_ZIP_SIZE / 1024 / 1024
            ),
        ));
    }

    let temp_dir = TempDir::new()
        .map_err(|e| AppError::internal(format!("Failed to create temp directory: {e}")))?;

    let temp_path = temp_dir.path().to_path_buf();
    let data = data.to_vec();

    tokio::task::spawn_blocking(move || {
        extract_zip_secure(&data, &temp_path)?;

        repomix::pack_directory(&temp_path, config)
            .map_err(|e| AppError::internal(format!("Failed to pack directory: {e}")))
    })
    .await
    .map_err(|e| AppError::internal(format!("Task join error: {e}")))?
}

/// Process a chunked upload by assembling chunks
async fn process_chunked_upload(
    state: Arc<AppState>,
    upload_id: Uuid,
    config: repomix::MergedConfig,
) -> Result<repomix::PackResult, AppError> {
    tracing::info!("Processing chunked upload: {}", upload_id);

    let assembled_path = crate::handlers::assemble_chunks(&state, upload_id).await?;

    let data = tokio::fs::read(&assembled_path)
        .await
        .map_err(|e| AppError::internal(format!("Failed to read assembled file: {e}")))?;

    let file_name = assembled_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("upload.zip")
        .to_string();

    state.cleanup_upload_session(upload_id).await;

    process_zip_file(&file_name, &data, config).await
}

/// Process a direct folder upload using browser-provided relative paths
async fn process_folder_upload(
    files: Vec<FolderUploadFile>,
    config: repomix::MergedConfig,
) -> Result<repomix::PackResult, AppError> {
    tracing::info!("Processing folder upload with {} files", files.len());

    let temp_dir = TempDir::new()
        .map_err(|e| AppError::internal(format!("Failed to create temp directory: {e}")))?;

    let temp_path = temp_dir.path().to_path_buf();

    tokio::task::spawn_blocking(move || {
        materialize_folder_upload(&files, &temp_path)?;

        repomix::pack_directory(&temp_path, config)
            .map_err(|e| AppError::internal(format!("Failed to pack directory: {e}")))
    })
    .await
    .map_err(|e| AppError::internal(format!("Task join error: {e}")))?
}

fn materialize_folder_upload(files: &[FolderUploadFile], dest: &Path) -> Result<(), AppError> {
    if files.is_empty() {
        return Err(AppError::bad_request("Folder upload is empty"));
    }

    if files.len() > MAX_FILES {
        return Err(AppError::new(
            StatusCode::PAYLOAD_TOO_LARGE,
            format!(
                "Folder contains too many files: {} > {}",
                files.len(),
                MAX_FILES
            ),
        ));
    }

    let total_size: usize = files.iter().map(|file| file.data.len()).sum();
    if total_size > MAX_DIRECT_UPLOAD_SIZE {
        return Err(AppError::new(
            StatusCode::PAYLOAD_TOO_LARGE,
            format!(
                "Folder upload exceeds maximum of {}MB",
                MAX_DIRECT_UPLOAD_SIZE / 1024 / 1024
            ),
        ));
    }

    std::fs::create_dir_all(dest)
        .map_err(|e| AppError::internal(format!("Failed to create destination: {e}")))?;

    let mut processed_paths = HashSet::new();

    for file in files {
        let normalized_path = normalize_relative_upload_path(&file.relative_path)?;
        let out_path = dest.join(&normalized_path);

        if !out_path.starts_with(dest) {
            return Err(AppError::bad_request(format!(
                "Path traversal detected: {}",
                file.relative_path
            )));
        }

        if !processed_paths.insert(out_path.clone()) {
            return Err(AppError::bad_request(format!(
                "Duplicate file path detected: {}",
                file.relative_path
            )));
        }

        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| AppError::internal(format!("Failed to create directory: {e}")))?;
        }

        std::fs::write(&out_path, &file.data)
            .map_err(|e| AppError::internal(format!("Failed to write uploaded file: {e}")))?;
    }

    Ok(())
}

/// Extract ZIP file with security checks
fn extract_zip_secure(data: &[u8], dest: &Path) -> Result<(), AppError> {
    use std::io::Cursor;
    use zip::ZipArchive;

    let cursor = Cursor::new(data);
    let mut archive = ZipArchive::new(cursor)
        .map_err(|e| AppError::bad_request(format!("Invalid ZIP file: {e}")))?;

    if archive.len() > MAX_FILES {
        return Err(AppError::new(
            StatusCode::PAYLOAD_TOO_LARGE,
            format!(
                "ZIP contains too many files: {} > {}",
                archive.len(),
                MAX_FILES
            ),
        ));
    }

    let mut total_uncompressed: u64 = 0;
    for i in 0..archive.len() {
        let file = archive
            .by_index(i)
            .map_err(|e| AppError::bad_request(format!("Failed to read ZIP entry: {e}")))?;
        total_uncompressed += file.size();
    }

    if total_uncompressed > MAX_UNCOMPRESSED_SIZE {
        return Err(AppError::new(
            StatusCode::PAYLOAD_TOO_LARGE,
            format!(
                "Uncompressed size exceeds maximum: {} > {}",
                total_uncompressed, MAX_UNCOMPRESSED_SIZE
            ),
        ));
    }

    let compression_ratio = total_uncompressed as f64 / data.len() as f64;
    if compression_ratio > MAX_COMPRESSION_RATIO {
        return Err(AppError::bad_request(format!(
            "Suspicious compression ratio: {:.2} > {}",
            compression_ratio, MAX_COMPRESSION_RATIO
        )));
    }

    std::fs::create_dir_all(dest)
        .map_err(|e| AppError::internal(format!("Failed to create destination: {e}")))?;

    let mut processed_paths = HashSet::new();

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| AppError::bad_request(format!("Failed to read ZIP entry: {e}")))?;

        let file_path = file.name().to_string();

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

        validate_path_limits(&file_path)?;

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
                "Path traversal detected: {}",
                file_path
            )));
        }

        if !processed_paths.insert(out_path.clone()) {
            return Err(AppError::bad_request(format!(
                "Duplicate file path detected: {}. This could indicate a malicious archive.",
                file_path
            )));
        }

        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| AppError::internal(format!("Failed to create directory: {e}")))?;
        }

        let mut out_file = std::fs::File::create(&out_path)
            .map_err(|e| AppError::internal(format!("Failed to create file: {e}")))?;

        std::io::copy(&mut file, &mut out_file)
            .map_err(|e| AppError::internal(format!("Failed to extract file: {e}")))?;
    }

    Ok(())
}

fn validate_path_limits(path: &str) -> Result<(), AppError> {
    if path.len() > MAX_PATH_LENGTH {
        return Err(AppError::bad_request(format!(
            "File path too long: {} > {}",
            path.len(),
            MAX_PATH_LENGTH
        )));
    }

    let nesting_level = path.matches('/').count();
    if nesting_level > MAX_NESTING_LEVEL {
        return Err(AppError::bad_request(format!(
            "Path nesting too deep: {} > {}",
            nesting_level, MAX_NESTING_LEVEL
        )));
    }

    Ok(())
}

fn normalize_relative_upload_path(path: &str) -> Result<PathBuf, AppError> {
    validate_path_limits(path)?;

    let raw_path = Path::new(path);
    if raw_path.is_absolute() {
        return Err(AppError::bad_request(format!(
            "Path traversal detected: {path}"
        )));
    }

    let mut normalized = PathBuf::new();

    for component in raw_path.components() {
        match component {
            Component::Normal(part) => normalized.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::Prefix(_) | Component::RootDir => {
                return Err(AppError::bad_request(format!(
                    "Path traversal detected: {path}"
                )));
            }
        }
    }

    if normalized.as_os_str().is_empty() {
        return Err(AppError::bad_request(format!(
            "Invalid uploaded file path: {path}"
        )));
    }

    Ok(normalized)
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
fn build_top_files(result: &repomix::PackResult) -> Vec<TopFile> {
    result
        .metrics
        .file_char_counts
        .iter()
        .take(10)
        .map(|metrics| TopFile {
            path: metrics.path.clone(),
            char_count: metrics.characters,
            token_count: metrics.tokens,
        })
        .collect()
}

fn build_all_files(result: &repomix::PackResult) -> Option<Vec<FileInfo>> {
    if result.metrics.file_char_counts.len() > MAX_BROWSER_FILE_SELECTION_FILES {
        return None;
    }

    Some(
        result
            .metrics
            .file_char_counts
            .iter()
            .map(|metrics| FileInfo {
                path: metrics.path.clone(),
                char_count: metrics.characters,
                token_count: metrics.tokens,
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::{
        build_all_files, build_pack_response, extract_zip_secure, materialize_folder_upload,
        FolderUploadFile, MAX_BROWSER_FILE_SELECTION_FILES,
    };
    use axum::http::StatusCode;
    use repomix::{core::metrics::PackMetrics, OutputStyle, PackResult};
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

    #[test]
    fn rejects_parent_dir_in_folder_uploads() {
        let dest = tempdir().unwrap();
        let files = vec![FolderUploadFile {
            relative_path: "../escape.txt".into(),
            data: b"owned".to_vec(),
        }];

        let err = materialize_folder_upload(&files, dest.path()).unwrap_err();

        assert_eq!(err.status, StatusCode::BAD_REQUEST);
        assert!(err.message.contains("Path traversal detected"));
    }

    #[test]
    fn writes_folder_uploads_with_relative_paths() {
        let dest = tempdir().unwrap();
        let files = vec![
            FolderUploadFile {
                relative_path: "demo/src/main.rs".into(),
                data: b"fn main() {}".to_vec(),
            },
            FolderUploadFile {
                relative_path: "demo/Cargo.toml".into(),
                data: b"[package]".to_vec(),
            },
        ];

        materialize_folder_upload(&files, dest.path()).unwrap();

        assert_eq!(
            std::fs::read_to_string(dest.path().join("demo/src/main.rs")).unwrap(),
            "fn main() {}"
        );
        assert_eq!(
            std::fs::read_to_string(dest.path().join("demo/Cargo.toml")).unwrap(),
            "[package]"
        );
    }

    #[test]
    fn builds_all_files_and_response_metadata() {
        let result = PackResult {
            content: "output".to_string(),
            metrics: PackMetrics {
                total_files: 2,
                total_characters: 42,
                total_tokens: 12,
                file_char_counts: vec![
                    repomix::core::metrics::FileMetrics {
                        path: "demo/src/main.rs".to_string(),
                        characters: 30,
                        tokens: 9,
                    },
                    repomix::core::metrics::FileMetrics {
                        path: "demo/Cargo.toml".to_string(),
                        characters: 12,
                        tokens: 3,
                    },
                ],
            },
            format: OutputStyle::Xml,
            file_paths: vec![
                "demo/src/main.rs".to_string(),
                "demo/Cargo.toml".to_string(),
            ],
        };

        let all_files =
            build_all_files(&result).expect("small packs should expose selectable files");
        assert_eq!(all_files.len(), 2);
        assert_eq!(all_files[0].path, "demo/src/main.rs");
        assert_eq!(all_files[0].token_count, 9);

        let response = build_pack_response(result, "xml".to_string(), Some("demo".to_string()));
        let metadata = response.metadata;

        assert_eq!(metadata.repository, "demo");
        assert_eq!(metadata.summary.unwrap().total_tokens, 12);
        assert_eq!(metadata.top_files.unwrap().len(), 2);
        assert_eq!(metadata.all_files.unwrap().len(), 2);
    }

    #[test]
    fn omits_all_files_when_pack_is_too_large_for_browser_selection() {
        let result = PackResult {
            content: "output".to_string(),
            metrics: PackMetrics {
                total_files: MAX_BROWSER_FILE_SELECTION_FILES + 1,
                total_characters: MAX_BROWSER_FILE_SELECTION_FILES + 1,
                total_tokens: MAX_BROWSER_FILE_SELECTION_FILES + 1,
                file_char_counts: (0..=MAX_BROWSER_FILE_SELECTION_FILES)
                    .map(|index| repomix::core::metrics::FileMetrics {
                        path: format!("demo/src/file-{index}.rs"),
                        characters: 1,
                        tokens: 1,
                    })
                    .collect(),
            },
            format: OutputStyle::Xml,
            file_paths: vec!["demo/src/file-0.rs".to_string()],
        };

        assert!(build_all_files(&result).is_none());

        let response = build_pack_response(result, "xml".to_string(), Some("demo".to_string()));
        assert!(response.metadata.all_files.is_none());
    }
}
