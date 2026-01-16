// filepath: /mailora-hub-imap/mailora-hub-imap/tests/integration_events.rs
use crate::main;
use axum::http::StatusCode;
use axum::Router;
use hyper::Body;
use tower::ServiceExt; // for `app.oneshot()` // assuming main.rs contains the setup for the app

#[tokio::test]
async fn test_events_endpoint() {
    let app = Router::new().nest("/events", main::app()); // Adjust this line based on your app structure

    // Test a GET request to /events
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/events")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Additional assertions can be added here based on expected response
}
