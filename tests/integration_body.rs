// filepath: /mailora-hub-imap/mailora-hub-imap/tests/integration_body.rs
use axum::{Router, routing::get, http::StatusCode};
use hyper::Body;
use tower::ServiceExt; // for `app.oneshot()`
use crate::app; // assuming you have a module that sets up your app

#[tokio::test]
async fn test_body_endpoint() {
    let app = Router::new().route("/body", get(app::body_handler)); // replace with your actual handler

    let response = app.oneshot(
        http::Request::builder()
            .method("GET")
            .uri("/body")
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    // Add more assertions based on expected response
}