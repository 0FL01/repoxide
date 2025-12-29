//! Application state management
//!
//! This module manages upload sessions and other shared state.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::Instant;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Application state shared across handlers
pub struct AppState {
    /// Active upload sessions
    pub uploads: RwLock<HashMap<Uuid, UploadSession>>,
}

/// Upload session tracking
#[derive(Debug)]
pub struct UploadSession {
    /// Unique session ID
    pub id: Uuid,
    /// Original file name
    pub file_name: String,
    /// Total file size in bytes
    pub file_size: u64,
    /// Total number of chunks expected
    pub total_chunks: u32,
    /// Set of received chunk indices
    pub received_chunks: HashSet<u32>,
    /// Temporary directory for chunk storage
    pub temp_dir: PathBuf,
    /// Session creation time
    pub created_at: Instant,
    /// Session expiration time
    pub expires_at: Instant,
}

impl AppState {
    /// Create a new application state
    pub fn new() -> Self {
        Self {
            uploads: RwLock::new(HashMap::new()),
        }
    }

    /// Clean up expired upload sessions
    pub async fn cleanup_expired(&self) {
        let mut uploads = self.uploads.write().await;
        let now = Instant::now();

        uploads.retain(|id, session| {
            if now > session.expires_at {
                tracing::info!("Cleaning up expired upload session: {}", id);
                // Clean up temp directory
                if session.temp_dir.exists() {
                    if let Err(e) = std::fs::remove_dir_all(&session.temp_dir) {
                        tracing::warn!("Failed to remove temp dir: {}", e);
                    }
                }
                false
            } else {
                true
            }
        });

        if !uploads.is_empty() {
            tracing::debug!("Active upload sessions: {}", uploads.len());
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl UploadSession {
    /// Create a new upload session
    pub fn new(
        file_name: String,
        file_size: u64,
        total_chunks: u32,
        temp_dir: PathBuf,
        ttl_secs: u64,
    ) -> Self {
        let now = Instant::now();
        Self {
            id: Uuid::new_v4(),
            file_name,
            file_size,
            total_chunks,
            received_chunks: HashSet::new(),
            temp_dir,
            created_at: now,
            expires_at: now + std::time::Duration::from_secs(ttl_secs),
        }
    }

    /// Check if all chunks have been received
    pub fn is_complete(&self) -> bool {
        self.received_chunks.len() == self.total_chunks as usize
    }

    /// Get upload progress (0.0 to 1.0)
    pub fn progress(&self) -> f64 {
        if self.total_chunks == 0 {
            return 0.0;
        }
        self.received_chunks.len() as f64 / self.total_chunks as f64
    }
}
