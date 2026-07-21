use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::domains::DomainRepository;
use crate::mailboxes::MailboxRepository;
use crate::rbac::authorization::AuthorizationService;
use crate::rbac::AuthUser;
use crate::users::UserRepository;

#[derive(Debug, Deserialize)]
pub struct UpdateUserRoleRequest {
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct MailboxAssignmentRequest {
    pub user_id: i64,
    pub can_view: bool,
    pub can_read: bool,
    pub can_reply: bool,
    pub can_send: bool,
    pub can_send_as: bool,
    pub can_mark_read: bool,
    pub can_move: bool,
    pub can_delete: bool,
    pub can_manage: bool,
}

#[derive(Debug, Serialize)]
pub struct MyPermissionsResponse {
    pub user_id: i64,
    pub role: String,
    pub is_super_admin: bool,
    pub allowed_mailbox_ids: Vec<i64>,
}

/// GET /api/v1/rbac/me/permissions
async fn get_my_permissions(
    auth_user: AuthUser,
    State(pool): State<SqlitePool>,
) -> impl IntoResponse {
    let auth_svc = AuthorizationService::new(&pool);
    let is_super = auth_svc.is_super_admin(auth_user.id).await.unwrap_or(false);
    let allowed = auth_svc
        .get_allowed_mailbox_ids(auth_user.id)
        .await
        .unwrap_or_default();

    (
        StatusCode::OK,
        Json(MyPermissionsResponse {
            user_id: auth_user.id,
            role: auth_user.role,
            is_super_admin: is_super,
            allowed_mailbox_ids: allowed,
        }),
    )
        .into_response()
}

/// POST /api/v1/rbac/users/:user_id/role (SuperAdmin only)
async fn update_user_role(
    auth_user: AuthUser,
    State(pool): State<SqlitePool>,
    Path(user_id): Path<i64>,
    Json(req): Json<UpdateUserRoleRequest>,
) -> impl IntoResponse {
    let auth_svc = AuthorizationService::new(&pool);
    match auth_svc.is_super_admin(auth_user.id).await {
        Ok(true) => {}
        _ => return (StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": "SuperAdmin rights required" }))).into_response(),
    }

    let repo = UserRepository::new(&pool);
    match repo.update_role(user_id, &req.role).await {
        Ok(u) => (StatusCode::OK, Json(serde_json::json!({ "ok": true, "user": u }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

/// POST /api/v1/rbac/domains/:domain_id/admins/:user_id (SuperAdmin only)
async fn assign_domain_admin(
    auth_user: AuthUser,
    State(pool): State<SqlitePool>,
    Path((domain_id, user_id)): Path<(i64, i64)>,
) -> impl IntoResponse {
    let auth_svc = AuthorizationService::new(&pool);
    match auth_svc.is_super_admin(auth_user.id).await {
        Ok(true) => {}
        _ => return (StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": "SuperAdmin rights required" }))).into_response(),
    }

    let repo = DomainRepository::new(&pool);
    match repo.assign_admin(user_id, domain_id, Some(auth_user.id)).await {
        Ok(assign) => (StatusCode::OK, Json(serde_json::json!({ "ok": true, "assignment": assign }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

/// DELETE /api/v1/rbac/domains/:domain_id/admins/:user_id (SuperAdmin only)
async fn revoke_domain_admin(
    auth_user: AuthUser,
    State(pool): State<SqlitePool>,
    Path((domain_id, user_id)): Path<(i64, i64)>,
) -> impl IntoResponse {
    let auth_svc = AuthorizationService::new(&pool);
    match auth_svc.is_super_admin(auth_user.id).await {
        Ok(true) => {}
        _ => return (StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": "SuperAdmin rights required" }))).into_response(),
    }

    let repo = DomainRepository::new(&pool);
    match repo.revoke_admin(user_id, domain_id).await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "ok": true }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

/// POST /api/v1/rbac/mailboxes/:mailbox_id/assignments (SuperAdmin or DomainAdmin for mailbox domain)
async fn assign_mailbox_user(
    auth_user: AuthUser,
    State(pool): State<SqlitePool>,
    Path(mailbox_id): Path<i64>,
    Json(req): Json<MailboxAssignmentRequest>,
) -> impl IntoResponse {
    let mb_repo = MailboxRepository::new(&pool);
    let mb = match mb_repo.get_by_id(mailbox_id).await {
        Ok(m) => m,
        Err(_) => return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Mailbox not found" }))).into_response(),
    };

    let auth_svc = AuthorizationService::new(&pool);
    match auth_svc.can_manage_domain(auth_user.id, mb.domain_id).await {
        Ok(true) => {}
        _ => return (StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": "Domain management rights required" }))).into_response(),
    }

    let perms = (
        req.can_view,
        req.can_read,
        req.can_reply,
        req.can_send,
        req.can_send_as,
        req.can_mark_read,
        req.can_move,
        req.can_delete,
        req.can_manage,
    );

    match mb_repo.assign_user(req.user_id, mailbox_id, perms, Some(auth_user.id)).await {
        Ok(assign) => (StatusCode::OK, Json(serde_json::json!({ "ok": true, "assignment": assign }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub fn routes<S>(pool: &SqlitePool) -> Router<S>
where
    S: Send + Sync + Clone + 'static,
    SqlitePool: axum::extract::FromRef<S>,
{
    let _ = pool;
    Router::new()
        .route("/me/permissions", get(get_my_permissions))
        .route("/users/:user_id/role", post(update_user_role))
        .route("/domains/:domain_id/admins/:user_id", post(assign_domain_admin).delete(revoke_domain_admin))
        .route("/mailboxes/:mailbox_id/assignments", post(assign_mailbox_user))
}
