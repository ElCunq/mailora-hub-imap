use axum::{response::IntoResponse, routing::Router, Json};
use serde_json::json;
use std::sync::Arc;

#[derive(Clone)]
pub struct EventStream {}

impl EventStream {
    pub fn new() -> Self {
        EventStream {}
    }

    pub async fn handle_events(self: Arc<Self>) -> impl IntoResponse {
        Json(json!({
            "message": "Events handled successfully"
        }))
    }
}

pub fn create_routes(event_stream: Arc<EventStream>) -> Router {
    Router::new().route(
        "/events",
        axum::routing::get({
            let es = event_stream.clone();
            move || {
                let es = es.clone();
                async move { es.handle_events().await }
            }
        }),
    )
}
