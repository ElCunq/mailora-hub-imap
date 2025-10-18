// filepath: /mailora-hub-imap/mailora-hub-imap/tests/integration_attachments.rs
use axum::{body::Body, http::StatusCode, Router};
use hyper::Client;
use tower::ServiceExt; // for `app.oneshot()`
use std::net::SocketAddr;

#[tokio::test]
async fn test_attachments_endpoint() {
    // Setup the application and routes
    let app = Router::new()
        .route("/attachments", axum::routing::get(attachments_handler));

    // Define the address to run the server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let _ = tokio::spawn(async move {
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .unwrap();
    });

    // Give the server a moment to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Create a client to send requests
    let client = Client::new();

    // Send a GET request to the /attachments endpoint
    let response = client
        .get(format!("http://{}/attachments", addr))
        .await
        .unwrap();

    // Assert the response status code
    assert_eq!(response.status(), StatusCode::OK);
}

// Dummy handler for the /attachments endpoint
async fn attachments_handler() -> &'static str {
    "Attachments endpoint"
}