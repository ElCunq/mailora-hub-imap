use axum::{
    extract::{Json, State},
    response::IntoResponse,
    routing::post,
    Router,
};
use serde_json::json;
use lettre::{Message, SmtpTransport, Transport};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    smtp_client: Arc<SmtpTransport>,
}

pub async fn send_email(State(state): State<AppState>, Json(payload): Json<EmailPayload>) -> impl IntoResponse {
    let email = Message::builder()
        .from(payload.from.parse().unwrap())
        .to(payload.to.parse().unwrap())
        .subject(payload.subject)
        .body(payload.body)
        .unwrap();

    match state.smtp_client.send(&email) {
        Ok(_) => (axum::http::StatusCode::OK, Json(json!({"status": "Email sent"}))),
        Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
    }
}

#[derive(serde::Deserialize)]
pub struct EmailPayload {
    from: String,
    to: String,
    subject: String,
    body: String,
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/action", post(send_email))
        .with_state(state)
}