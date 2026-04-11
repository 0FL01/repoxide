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
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;

use crate::{
    error::AppError,
    state::{AppState, UploadSession},
    types::{ChunkResponse, InitUploadRequest, InitUploadResponse, StatusResponse},
};

/// Configuration constants for chunked uploads
const MAX_FILE_SIZE: u64 = 50 * 1024 * 1024; // 50MB
const MAX_CONCURRENT_UPLOADS: usize = 100;
const UPLOAD_TTL_SECS: u64 = 60 * 60; // 1 hour

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
    // Validate file size
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

    // Validate file name
    if req.file_name.is_empty() || !req.file_name.ends_with(".zip") {
        return Err(AppError::bad_request("Only ZIP files are allowed"));
    }

    // Check concurrent uploads limit
    let uploads_count = state.uploads.read().await.len();
    if uploads_count >= MAX_CONCURRENT_UPLOADS {
        return Err(AppError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "Too many concurrent uploads. Please try again later.",
        ));
    }

    // Validate total chunks
    if req.total_chunks == 0 {
        return Err(AppError::bad_request("Total chunks must be greater than 0"));
    }

    // Create temporary directory for chunks
    let temp_dir_handle = tempfile::Builder::new()
        .prefix(&format!("repomix-upload-{}-", Uuid::new_v4()))
        .tempdir()
        .map_err(|e| AppError::internal(format!("Failed to create temp directory: {}", e)))?;
    let temp_dir = temp_dir_handle.path().to_path_buf();
    // Keep the temp directory (don't delete on drop)
    let _ = temp_dir_handle.keep();

    // Create upload session
    let session = UploadSession::new(
        req.file_name.clone(),
        req.file_size,
        req.total_chunks,
        temp_dir,
        UPLOAD_TTL_SECS,
    );

    let upload_id = session.id;

    // Store session
    state.uploads.write().await.insert(upload_id, session);

    tracing::info!(
        upload_id = %upload_id,
        file_name = %req.file_name,
        file_size = req.file_size,
        total_chunks = req.total_chunks,
        "Chunked upload initialized"
    );

    Ok(Json(InitUploadResponse {
        upload_id,
        expires_in: UPLOAD_TTL_SECS,
    }))
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

    // Get session (with write lock for modification)
    let mut uploads = state.uploads.write().await;
    let session = uploads
        .get_mut(&upload_id)
        .ok_or_else(|| AppError::not_found("Upload session not found or expired"))?;

    // Check if session expired
    if Instant::now() > session.expires_at {
        // Clean up temp directory
        if session.temp_dir.exists() {
            let _ = std::fs::remove_dir_all(&session.temp_dir);
        }
        uploads.remove(&upload_id);
        return Err(AppError::new(
            StatusCode::GONE,
            "Upload session expired",
        ));
    }

    // Validate chunk index
    if chunk_index >= session.total_chunks {
        return Err(AppError::bad_request(format!(
            "Invalid chunk index: {}. Expected 0-{}",
            chunk_index,
            session.total_chunks - 1
        )));
    }

    // Skip if already received (idempotency)
    if !session.received_chunks.contains(&chunk_index) {
        // Write chunk to file
        let chunk_path = session
            .temp_dir
            .join(format!("chunk_{:06}", chunk_index));

        tokio::fs::write(&chunk_path, &body)
            .await
            .map_err(|e| AppError::internal(format!("Failed to write chunk: {}", e)))?;

        session.received_chunks.insert(chunk_index);

        tracing::debug!(
            upload_id = %upload_id,
            chunk_index,
            received = session.received_chunks.len(),
            total = session.total_chunks,
            "Chunk received"
        );
    }

    let chunks_received = session.received_chunks.len();
    let total_chunks = session.total_chunks;
    let complete = session.is_complete();

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
pub async fn assemble_chunks(
    state: &AppState,
    upload_id: Uuid,
) -> Result<PathBuf, AppError> {
    let uploads = state.uploads.read().await;
    let session = uploads
        .get(&upload_id)
        .ok_or_else(|| AppError::not_found("Upload session not found or expired"))?;

    // Check if all chunks received
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

    // Assemble file path
    let assembled_path = session.temp_dir.join(&session.file_name);

    // Read and concatenate all chunks in order
    let mut assembled_file = tokio::fs::File::create(&assembled_path)
        .await
        .map_err(|e| AppError::internal(format!("Failed to create assembled file: {}", e)))?;

    use tokio::io::AsyncWriteExt;

    for i in 0..session.total_chunks {
        let chunk_path = session.temp_dir.join(format!("chunk_{:06}", i));

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

    if metadata.len() != session.file_size {
        return Err(AppError::internal(format!(
            "File size mismatch. Expected {}, got {}",
            session.file_size,
            metadata.len()
        )));
    }

    tracing::info!(
        upload_id = %upload_id,
        file_name = %session.file_name,
        file_size = metadata.len(),
        "File assembled successfully"
    );

    Ok(assembled_path)
}
