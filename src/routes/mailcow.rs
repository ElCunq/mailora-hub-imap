use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{routing::{get, post}, Json, Router};
use sqlx::SqlitePool;
use crate::mailcow::models::{CreateMailcowInstance, UpdateMailcowInstance};
use crate::mailcow::MailcowRepository;
use crate::mailcow::discovery::DiscoveryService;

pub fn routes<S>(_pool: &SqlitePool) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    SqlitePool: axum::extract::FromRef<S>,
{
    Router::new()
        .route("/instances", get(list_instances).post(create_instance))
        .route(
            "/instances/:id",
            get(get_instance)
                .patch(update_instance)
                .delete(delete_instance),
        )
        .route("/discovery/run/:id", post(run_discovery))
        .route("/discovery/run-all", post(run_discovery_all))
}

async fn list_instances(State(pool): State<SqlitePool>) -> impl IntoResponse {
    let repo = MailcowRepository::new(&pool);
    match repo.list_all().await {
        Ok(list) => (
            StatusCode::OK,
            Json(serde_json::json!({ "ok": true, "data": list })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "ok": false, "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn create_instance(
    State(pool): State<SqlitePool>,
    Json(payload): Json<CreateMailcowInstance>,
) -> impl IntoResponse {
    let repo = MailcowRepository::new(&pool);
    match repo.create(payload).await {
        Ok(inst) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "ok": true, "data": inst })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "ok": false, "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn get_instance(State(pool): State<SqlitePool>, Path(id): Path<i64>) -> impl IntoResponse {
    let repo = MailcowRepository::new(&pool);
    match repo.get_by_id(id).await {
        Ok(inst) => (
            StatusCode::OK,
            Json(serde_json::json!({ "ok": true, "data": inst })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "ok": false, "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn update_instance(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateMailcowInstance>,
) -> impl IntoResponse {
    let repo = MailcowRepository::new(&pool);
    match repo.update(id, payload).await {
        Ok(inst) => (
            StatusCode::OK,
            Json(serde_json::json!({ "ok": true, "data": inst })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "ok": false, "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn delete_instance(State(pool): State<SqlitePool>, Path(id): Path<i64>) -> impl IntoResponse {
    let repo = MailcowRepository::new(&pool);
    match repo.delete(id).await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "ok": true }))).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "ok": false, "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn run_discovery(State(pool): State<SqlitePool>, Path(id): Path<i64>) -> impl IntoResponse {
    let service = DiscoveryService::new(&pool);
    match service.run_discovery_for_instance(id).await {
        Ok(summary) => (
            StatusCode::OK,
            Json(serde_json::json!({ "ok": true, "data": summary })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "ok": false, "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn run_discovery_all(State(pool): State<SqlitePool>) -> impl IntoResponse {
    let service = DiscoveryService::new(&pool);
    match service.run_discovery_all().await {
        Ok(summaries) => (
            StatusCode::OK,
            Json(serde_json::json!({ "ok": true, "data": summaries })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "ok": false, "error": e.to_string() })),
        )
            .into_response(),
    }
}
