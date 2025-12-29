//! Chunked upload handlers

use axum::{
    extract::{Path, State},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    error::AppError,
    state::AppState,
    types::{ChunkResponse, InitUploadRequest, InitUploadResponse, StatusResponse},
};

/// Initialize chunked upload (placeholder for Phase 4)
pub async fn upload_init(
    State(_state): State<Arc<AppState>>,
    Json(_req): Json<InitUploadRequest>,
) -> Result<Json<InitUploadResponse>, AppError> {
    // TODO: Phase 4 - Implement upload initialization
    Err(AppError::internal("Upload init not yet implemented"))
}

/// Upload a chunk (placeholder for Phase 4)
pub async fn upload_chunk(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<ChunkResponse>, AppError> {
    // TODO: Phase 4 - Implement chunk upload
    Err(AppError::internal("Upload chunk not yet implemented"))
}

/// Get upload status (placeholder for Phase 4)
pub async fn upload_status(
    Path(_id): Path<Uuid>,
    State(_state): State<Arc<AppState>>,
) -> Result<Json<StatusResponse>, AppError> {
    // TODO: Phase 4 - Implement status query
    Err(AppError::internal("Upload status not yet implemented"))
}
