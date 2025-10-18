// filepath: /mailora-hub-imap/mailora-hub-imap/tests/integration_action.rs
use axum::{http::StatusCode, Router};
use hyper::Body;
use tower::ServiceExt; // for `app.oneshot()`
use mailora_hub_imap::main; // assuming main.rs contains the app setup

#[tokio::test]
async fn test_action_send() {
    let app = Router::new().nest("/", main().await);

    let response = app
        .oneshot(
            http::Request::builder()
                .method("POST")
                .uri("/action")
                .body(Body::from("test payload"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    // Additional assertions can be added based on expected behavior
}