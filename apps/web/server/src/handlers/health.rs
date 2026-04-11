//! Health check handler

use axum::Json;

/// Health check endpoint
///
/// Returns "OK" to indicate the server is running
pub async fn health() -> Json<String> {
    Json("OK".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health() {
        let response = health().await;
        assert_eq!(response.0, "OK");
    }
}
