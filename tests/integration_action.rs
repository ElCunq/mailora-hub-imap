use axum::{body::Body, http::{Request, StatusCode}, routing::get, Router};
use tower::ServiceExt;

#[tokio::test]
async fn test_healthz_and_folders() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .unwrap();

    let app = Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .merge(mailora_hub_imap::routes::routes(&pool))
        .with_state(pool.clone());

    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/healthz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let response_folders = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/folders?accountId=nonexistent")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response_folders.status(), StatusCode::NOT_FOUND);
}
