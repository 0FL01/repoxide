//! Chunked upload handlers
//!
//! Implements chunked file upload functionality for large files.
//! Files are split into chunks on the client side and uploaded sequentially.

use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use std::path::{Path as FsPath, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;

use crate::{
    error::AppError,
    state::{AppState, UploadSession},
    types::{ChunkResponse, InitUploadRequest, InitUploadResponse, StatusResponse},
};

/// Configuration constants for chunked uploads
const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024; // 100MB
const MAX_CONCURRENT_UPLOADS: usize = 4;
const MAX_RESERVED_UPLOAD_SESSIONS: usize = 64;
const MAX_COMPLETED_UPLOAD_SESSIONS: usize = 4;
const MAX_FILE_NAME_LENGTH: usize = 255;
const MAX_TOTAL_CHUNKS: u32 = 1024;
const INIT_RESERVATION_TTL_SECS: u64 = 2 * 60;
const COMPLETED_UPLOAD_TTL_SECS: u64 = 5 * 60;
const ACTIVE_UPLOAD_TTL_SECS: u64 = 60 * 60; // 1 hour

/// Query parameters for chunk upload
#[derive(Debug, Deserialize)]
pub struct ChunkQuery {
    #[serde(rename = "uploadId")]
    upload_id: Uuid,
    #[serde(rename = "chunkIndex")]
    chunk_index: u32,
}

/// Initialize chunked upload session
///
/// POST /api/upload/init
///
/// Creates a new upload session and returns an upload ID.
/// The client should then upload chunks using the upload_chunk endpoint.
pub async fn upload_init(
    State(state): State<Arc<AppState>>,
    Json(req): Json<InitUploadRequest>,
) -> Result<Json<InitUploadResponse>, AppError> {
    state.cleanup_expired().await;

    // Validate file size
    if req.file_size == 0 {
        return Err(AppError::bad_request("File size must be greater than 0"));
    }

    if req.file_size > MAX_FILE_SIZE {
        return Err(AppError::new(
            StatusCode::PAYLOAD_TOO_LARGE,
            format!(
                "File size {:.2}MB exceeds maximum limit of {}MB",
                req.file_size as f64 / 1024.0 / 1024.0,
                MAX_FILE_SIZE / 1024 / 1024
            ),
        ));
    }

    let file_name = sanitize_upload_file_name(&req.file_name)?;

    let uploads = state.uploads.read().await;
    let reserved_uploads = uploads
        .values()
        .filter(|session| !session.has_started())
        .count();
    let completed_uploads = uploads
        .values()
        .filter(|session| session.is_complete())
        .count();
    drop(uploads);

    if reserved_uploads >= MAX_RESERVED_UPLOAD_SESSIONS {
        return Err(AppError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "Too many pending upload sessions. Please try again shortly.",
        ));
    }

    if completed_uploads >= MAX_COMPLETED_UPLOAD_SESSIONS {
        return Err(AppError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "Too many completed uploads are awaiting processing. Finish or retry shortly.",
        ));
    }

    // Validate total chunks
    if req.total_chunks == 0 {
        return Err(AppError::bad_request("Total chunks must be greater than 0"));
    }

    if req.total_chunks > MAX_TOTAL_CHUNKS {
        return Err(AppError::bad_request(format!(
            "Total chunks exceeds maximum limit of {MAX_TOTAL_CHUNKS}"
        )));
    }

    if u64::from(req.total_chunks) > req.file_size {
        return Err(AppError::bad_request(
            "Total chunks cannot exceed declared file size",
        ));
    }

    let temp_dir = std::env::temp_dir().join(format!("repomix-upload-{}", Uuid::new_v4()));

    // Create upload session
    let session = UploadSession::new(
        file_name.clone(),
        req.file_size,
        req.total_chunks,
        temp_dir,
        INIT_RESERVATION_TTL_SECS,
    );

    let upload_id = session.id;

    // Store session
    state.uploads.write().await.insert(upload_id, session);

    tracing::info!(
        upload_id = %upload_id,
        file_name = %file_name,
        file_size = req.file_size,
        total_chunks = req.total_chunks,
        "Chunked upload initialized"
    );

    Ok(Json(InitUploadResponse {
        upload_id,
        expires_in: INIT_RESERVATION_TTL_SECS,
    }))
}

fn sanitize_upload_file_name(file_name: &str) -> Result<String, AppError> {
    let trimmed = file_name.trim();
    if trimmed.is_empty() {
        return Err(AppError::bad_request("Only ZIP files are allowed"));
    }

    let base_name = FsPath::new(trimmed)
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| AppError::bad_request("Invalid upload file name"))?;

    if base_name != trimmed {
        return Err(AppError::bad_request(
            "Upload file name must not contain path segments",
        ));
    }

    if base_name.len() > MAX_FILE_NAME_LENGTH {
        return Err(AppError::bad_request("Upload file name is too long"));
    }

    if !base_name.to_ascii_lowercase().ends_with(".zip") {
        return Err(AppError::bad_request("Only ZIP files are allowed"));
    }

    Ok(base_name.to_string())
}

/// Upload a single chunk
///
/// POST /api/upload/chunk?uploadId={id}&chunkIndex={index}
///
/// The chunk data should be sent as the raw request body.
pub async fn upload_chunk(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ChunkQuery>,
    body: Bytes,
) -> Result<Json<ChunkResponse>, AppError> {
    let upload_id = query.upload_id;
    let chunk_index = query.chunk_index;

    // Validate chunk data
    if body.is_empty() {
        return Err(AppError::bad_request("Chunk data is required"));
    }

    let (temp_dir, chunk_path) = {
        let mut uploads = state.uploads.write().await;
        let active_uploads = uploads
            .values()
            .filter(|session| session.holds_active_slot())
            .count();
        let session = uploads
            .get_mut(&upload_id)
            .ok_or_else(|| AppError::not_found("Upload session not found or expired"))?;

        if Instant::now() > session.expires_at {
            if session.temp_dir.exists() {
                let _ = std::fs::remove_dir_all(&session.temp_dir);
            }
            uploads.remove(&upload_id);
            return Err(AppError::new(StatusCode::GONE, "Upload session expired"));
        }

        if chunk_index >= session.total_chunks {
            return Err(AppError::bad_request(format!(
                "Invalid chunk index: {}. Expected 0-{}",
                chunk_index,
                session.total_chunks - 1
            )));
        }

        if !session.has_started() && active_uploads >= MAX_CONCURRENT_UPLOADS {
            return Err(AppError::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "Too many active uploads. Please try again later.",
            ));
        }

        if session.received_chunks.contains(&chunk_index)
            || session.pending_chunks.contains(&chunk_index)
        {
            let chunks_received = session.received_chunks.len();
            let total_chunks = session.total_chunks;
            let complete = session.is_complete();

            return Ok(Json(ChunkResponse {
                upload_id,
                chunks_received,
                total_chunks,
                complete,
            }));
        }

        session.pending_chunks.insert(chunk_index);
        session.refresh_expiry(ACTIVE_UPLOAD_TTL_SECS);

        let temp_dir = session.temp_dir.clone();
        let chunk_path = temp_dir.join(format!("chunk_{:06}", chunk_index));

        (temp_dir, chunk_path)
    };

    if let Err(e) = tokio::fs::create_dir_all(&temp_dir).await {
        if let Some(session) = state.uploads.write().await.get_mut(&upload_id) {
            session.pending_chunks.remove(&chunk_index);
            session.refresh_expiry(INIT_RESERVATION_TTL_SECS);
        }

        return Err(AppError::internal(format!(
            "Failed to create upload temp directory: {}",
            e
        )));
    }

    if let Err(e) = tokio::fs::write(&chunk_path, &body).await {
        if let Some(session) = state.uploads.write().await.get_mut(&upload_id) {
            session.pending_chunks.remove(&chunk_index);
            if session.has_started() {
                session.refresh_expiry(ACTIVE_UPLOAD_TTL_SECS);
            } else {
                session.refresh_expiry(INIT_RESERVATION_TTL_SECS);
            }
        }
        return Err(AppError::internal(format!("Failed to write chunk: {}", e)));
    }

    let chunk_size = body.len() as u64;
    let mut uploads = state.uploads.write().await;
    let (
        chunks_received,
        total_chunks,
        complete,
        size_mismatch,
        received_bytes,
        file_size,
        temp_dir,
    ) = {
        let session = uploads
            .get_mut(&upload_id)
            .ok_or_else(|| AppError::not_found("Upload session not found or expired"))?;

        if Instant::now() > session.expires_at {
            if session.temp_dir.exists() {
                let _ = std::fs::remove_dir_all(&session.temp_dir);
            }
            uploads.remove(&upload_id);
            return Err(AppError::new(StatusCode::GONE, "Upload session expired"));
        }

        session.pending_chunks.remove(&chunk_index);
        if session.has_started() {
            session.refresh_expiry(ACTIVE_UPLOAD_TTL_SECS);
        } else {
            session.refresh_expiry(INIT_RESERVATION_TTL_SECS);
        }

        if session.received_bytes.saturating_add(chunk_size) > session.file_size {
            let _ = std::fs::remove_file(&chunk_path);
            return Err(AppError::bad_request(
                "Chunk data exceeds declared upload size",
            ));
        }

        session.received_chunks.insert(chunk_index);
        session.received_bytes += chunk_size;
        if session.is_complete() {
            session.refresh_expiry(COMPLETED_UPLOAD_TTL_SECS);
        } else {
            session.refresh_expiry(ACTIVE_UPLOAD_TTL_SECS);
        }

        tracing::debug!(
            upload_id = %upload_id,
            chunk_index,
            received = session.received_chunks.len(),
            total = session.total_chunks,
            "Chunk received"
        );

        let chunks_received = session.received_chunks.len();
        let total_chunks = session.total_chunks;
        let complete = session.is_complete();
        let size_mismatch = complete && session.received_bytes != session.file_size;

        (
            chunks_received,
            total_chunks,
            complete,
            size_mismatch,
            session.received_bytes,
            session.file_size,
            session.temp_dir.clone(),
        )
    };

    if size_mismatch {
        if temp_dir.exists() {
            let _ = std::fs::remove_dir_all(&temp_dir);
        }
        uploads.remove(&upload_id);
        return Err(AppError::bad_request(format!(
            "Uploaded bytes {} do not match declared file size {}",
            received_bytes, file_size
        )));
    }

    if complete {
        tracing::info!(
            upload_id = %upload_id,
            "Upload complete - all chunks received"
        );
    }

    Ok(Json(ChunkResponse {
        upload_id,
        chunks_received,
        total_chunks,
        complete,
    }))
}

/// Get upload status
///
/// GET /api/upload/status/{id}
///
/// Returns the current status of an upload session.
pub async fn upload_status(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<StatusResponse>, AppError> {
    let uploads = state.uploads.read().await;
    let session = uploads
        .get(&id)
        .ok_or_else(|| AppError::not_found("Upload session not found"))?;

    Ok(Json(StatusResponse {
        upload_id: session.id,
        chunks_received: session.received_chunks.len(),
        total_chunks: session.total_chunks,
        progress: session.progress(),
        complete: session.is_complete(),
    }))
}

/// Assemble chunks into a complete file
///
/// This function is called from the pack handler when processing an uploadId.
/// It reads all chunks in order and concatenates them into a single file.
pub async fn assemble_chunks(state: &AppState, upload_id: Uuid) -> Result<PathBuf, AppError> {
    let (total_chunks, file_name, file_size, temp_dir) = {
        let uploads = state.uploads.read().await;
        let session = uploads
            .get(&upload_id)
            .ok_or_else(|| AppError::not_found("Upload session not found or expired"))?;

        if !session.is_complete() {
            return Err(AppError::bad_request(format!(
                "Upload incomplete. Received {} of {} chunks",
                session.received_chunks.len(),
                session.total_chunks
            )));
        }

        tracing::info!(
            upload_id = %upload_id,
            total_chunks = session.total_chunks,
            file_name = %session.file_name,
            "Assembling file from chunks"
        );

        (
            session.total_chunks,
            session.file_name.clone(),
            session.file_size,
            session.temp_dir.clone(),
        )
    };

    let assembled_path = temp_dir.join(&file_name);

    // Read and concatenate all chunks in order
    let mut assembled_file = tokio::fs::File::create(&assembled_path)
        .await
        .map_err(|e| AppError::internal(format!("Failed to create assembled file: {}", e)))?;

    use tokio::io::AsyncWriteExt;

    for i in 0..total_chunks {
        let chunk_path = temp_dir.join(format!("chunk_{:06}", i));

        let chunk_data = tokio::fs::read(&chunk_path)
            .await
            .map_err(|e| AppError::internal(format!("Failed to read chunk {}: {}", i, e)))?;

        assembled_file
            .write_all(&chunk_data)
            .await
            .map_err(|e| AppError::internal(format!("Failed to write chunk {}: {}", i, e)))?;
    }

    assembled_file
        .flush()
        .await
        .map_err(|e| AppError::internal(format!("Failed to flush file: {}", e)))?;

    // Verify file size
    let metadata = tokio::fs::metadata(&assembled_path)
        .await
        .map_err(|e| AppError::internal(format!("Failed to get file metadata: {}", e)))?;

    if metadata.len() != file_size {
        return Err(AppError::internal(format!(
            "File size mismatch. Expected {}, got {}",
            file_size,
            metadata.len()
        )));
    }

    tracing::info!(
        upload_id = %upload_id,
        file_name = %file_name,
        file_size = metadata.len(),
        "File assembled successfully"
    );

    Ok(assembled_path)
}

#[cfg(test)]
mod tests {
    use super::sanitize_upload_file_name;
    use axum::http::StatusCode;

    #[test]
    fn accepts_plain_zip_file_name() {
        assert_eq!(
            sanitize_upload_file_name("archive.zip").unwrap(),
            "archive.zip"
        );
    }

    #[test]
    fn rejects_upload_file_name_with_path_segments() {
        let err = sanitize_upload_file_name("../../archive.zip").unwrap_err();

        assert_eq!(err.status, StatusCode::BAD_REQUEST);
        assert!(err.message.contains("path segments"));
    }

    #[test]
    fn rejects_non_zip_upload_file_name() {
        let err = sanitize_upload_file_name("archive.tar").unwrap_err();

        assert_eq!(err.status, StatusCode::BAD_REQUEST);
        assert!(err.message.contains("Only ZIP files are allowed"));
    }
}
