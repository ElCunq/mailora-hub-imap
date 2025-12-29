use axum::{
    extract::{State, Json, Path},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, patch, delete},
    Router,
};
use crate::models::user::User;
use crate::rbac::AdminUser;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct UpdateRoleReq {
    pub role: String,
}

#[derive(Deserialize)]
pub struct AssignAccountReq {
    pub account_id: String,
}

#[derive(Serialize)]
pub struct UserWithAccounts {
    pub user: User,
    pub accounts: Vec<String>,
}

async fn list_users(
    _admin: AdminUser,
    State(pool): State<sqlx::SqlitePool>,
) -> impl IntoResponse {
    match sqlx::query_as::<_, User>("SELECT * FROM users")
        .fetch_all(&pool)
        .await {
            Ok(users) => (StatusCode::OK, Json(users)).into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
        }
}

async fn update_user_role(
    _admin: AdminUser,
    State(pool): State<sqlx::SqlitePool>,
    Path(user_id): Path<i64>,
    Json(req): Json<UpdateRoleReq>,
) -> impl IntoResponse {
    match sqlx::query("UPDATE users SET role = ? WHERE id = ?")
        .bind(&req.role)
        .bind(user_id)
        .execute(&pool)
        .await {
            Ok(_) => StatusCode::OK.into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
        }
}

async fn assign_account(
    _admin: AdminUser,
    State(pool): State<sqlx::SqlitePool>,
    Path(user_id): Path<i64>,
    Json(req): Json<AssignAccountReq>,
) -> impl IntoResponse {
    match sqlx::query("INSERT OR IGNORE INTO user_accounts (user_id, account_id) VALUES (?, ?)")
        .bind(user_id)
        .bind(&req.account_id)
        .execute(&pool)
        .await {
            Ok(_) => StatusCode::CREATED.into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
        }
}

async fn unassign_account(
    _admin: AdminUser,
    State(pool): State<sqlx::SqlitePool>,
    Path((user_id, account_id)): Path<(i64, String)>,
) -> impl IntoResponse {
    match sqlx::query("DELETE FROM user_accounts WHERE user_id = ? AND account_id = ?")
        .bind(user_id)
        .bind(&account_id)
        .execute(&pool)
        .await {
            Ok(_) => StatusCode::OK.into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
        }
}

async fn list_user_accounts(
    _admin: AdminUser,
    State(pool): State<sqlx::SqlitePool>,
    Path(user_id): Path<i64>,
) -> impl IntoResponse {
    match sqlx::query_scalar::<_, String>("SELECT account_id FROM user_accounts WHERE user_id = ?")
        .bind(user_id)
        .fetch_all(&pool)
        .await {
            Ok(accounts) => (StatusCode::OK, Json(accounts)).into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
        }
}

pub fn router() -> Router<sqlx::SqlitePool> {
    Router::new()
        .route("/admin/users", get(list_users))
        .route("/admin/users/:user_id/role", patch(update_user_role))
        .route("/admin/users/:user_id/accounts", get(list_user_accounts).post(assign_account))
        .route("/admin/users/:user_id/accounts/:account_id", delete(unassign_account))
}
