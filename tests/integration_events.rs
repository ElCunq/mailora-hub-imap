use axum::{body::Body, http::{Request, StatusCode}, routing::get, Router};
use std::sync::Arc;
use tower::ServiceExt;

#[tokio::test]
async fn test_idle_status_endpoint() {
    let idle_manager = Arc::new(mailora_hub_imap::services::idle_watcher_service::IdleWatcherManager::new());
    let app = Router::new()
        .route("/idle/status", get(mailora_hub_imap::routes::idle::idle_status))
        .with_state(idle_manager.clone());

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/idle/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
