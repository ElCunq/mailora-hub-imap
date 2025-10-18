// filepath: /mailora-hub-imap/mailora-hub-imap/tests/integration_diff.rs
use axum::{Router, routing::get};
use hyper::Body;
use tower::ServiceExt; // for `app.oneshot()`
use crate::main; // assuming main.rs sets up the app

#[tokio::test]
async fn test_diff_endpoint() {
    let app = Router::new().route("/diff", get(main::diff_handler)); // replace with actual handler

    let response = app.oneshot(
        hyper::Request::builder()
            .method("GET")
            .uri("/diff")
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();

    assert_eq!(response.status(), 200); // adjust based on expected status
}