use axum::{body::Body, http::{Request, StatusCode}, Router};
use tower::ServiceExt;

#[tokio::test]
async fn test_attachments_endpoint_missing_account() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .unwrap();

    let app = Router::new()
        .merge(mailora_hub_imap::routes::routes(&pool))
        .with_state(pool.clone());

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/attachments?accountId=nonexistent&uid=1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}
