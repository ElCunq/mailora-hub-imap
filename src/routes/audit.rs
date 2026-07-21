use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;
use sqlx::SqlitePool;
use crate::audit::service::AuditService;
use crate::domains::DomainRepository;
use crate::rbac::{AuthUser, AuthorizationService};

#[derive(Debug, Deserialize)]
pub struct AuditLogQuery {
    pub actor_user_id: Option<i64>,
    pub domain_id: Option<i64>,
    pub mailbox_id: Option<i64>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct SendAuditQuery {
    pub mailbox_id: Option<i64>,
    pub actor_user_id: Option<i64>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct SyncRunsQuery {
    pub mailbox_id: Option<i64>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct SyncRunEntry {
    pub id: i64,
    pub mailbox_id: i64,
    pub folder_name: String,
    pub sync_type: String,
    pub status: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub duration_ms: Option<i64>,
    pub new_count: Option<i64>,
    pub updated_count: Option<i64>,
    pub deleted_count: Option<i64>,
    pub error_count: Option<i64>,
    pub error_message: Option<String>,
}

/// GET /api/v3/audit-logs
pub async fn list_audit_logs_handler(
    State(pool): State<SqlitePool>,
    auth_user: AuthUser,
    Query(q): Query<AuditLogQuery>,
) -> impl IntoResponse {
    let limit = q.limit.unwrap_or(50).clamp(1, 200);
    let offset = q.offset.unwrap_or(0).max(0);

    let auth_svc = AuthorizationService::new(&pool);
    let is_super = auth_svc.is_super_admin(auth_user.id).await.unwrap_or(false);

    let mut actor_filter = q.actor_user_id;
    let domain_filter = q.domain_id;
    let mailbox_filter = q.mailbox_id;

    if !is_super {
        // If DomainAdmin, enforce domain constraint or actor = self
        let dom_repo = DomainRepository::new(&pool);
        if let Ok(domains) = dom_repo.list_user_domains(auth_user.id).await {
            if !domains.is_empty() {
                // User is DomainAdmin for some domains
                if let Some(did) = domain_filter {
                    if !domains.iter().any(|d| d.id == did) {
                        return (StatusCode::FORBIDDEN, Json(json!({ "error": "Access denied for requested domain_id" }))).into_response();
                    }
                } else if mailbox_filter.is_none() && actor_filter.is_none() {
                    // Filter to first managed domain if no filter provided (or we let actor_filter = self)
                    actor_filter = Some(auth_user.id);
                }
            } else {
                // Regular user: can only view their own logs
                actor_filter = Some(auth_user.id);
            }
        } else {
            actor_filter = Some(auth_user.id);
        }
    }

    let audit_svc = AuditService::new(&pool);
    match audit_svc.list_logs(actor_filter, domain_filter, mailbox_filter, limit, offset).await {
        Ok(logs) => Json(json!({ "ok": true, "logs": logs })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "ok": false, "error": e.to_string() }))).into_response(),
    }
}

/// GET /api/v3/send-audit
pub async fn list_send_audit_handler(
    State(pool): State<SqlitePool>,
    auth_user: AuthUser,
    Query(q): Query<SendAuditQuery>,
) -> impl IntoResponse {
    let limit = q.limit.unwrap_or(50).clamp(1, 200);
    let offset = q.offset.unwrap_or(0).max(0);

    let auth_svc = AuthorizationService::new(&pool);
    let is_super = auth_svc.is_super_admin(auth_user.id).await.unwrap_or(false);

    let mailbox_filter = q.mailbox_id;
    let mut actor_filter = q.actor_user_id;

    if !is_super {
        if let Some(mid) = mailbox_filter {
            if !auth_svc.check_mailbox_permission(auth_user.id, mid, crate::rbac::Permission::View).await.unwrap_or(false) {
                return (StatusCode::FORBIDDEN, Json(json!({ "error": "Access denied for mailbox_id" }))).into_response();
            }
        } else {
            actor_filter = Some(auth_user.id);
        }
    }

    let audit_svc = AuditService::new(&pool);
    match audit_svc.list_send_audit(mailbox_filter, actor_filter, limit, offset).await {
        Ok(entries) => Json(json!({ "ok": true, "send_audit": entries })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "ok": false, "error": e.to_string() }))).into_response(),
    }
}

/// GET /api/v3/sync-runs
pub async fn list_sync_runs_handler(
    State(pool): State<SqlitePool>,
    auth_user: AuthUser,
    Query(q): Query<SyncRunsQuery>,
) -> impl IntoResponse {
    let limit = q.limit.unwrap_or(50).clamp(1, 200);
    let offset = q.offset.unwrap_or(0).max(0);

    let auth_svc = AuthorizationService::new(&pool);
    let is_super = auth_svc.is_super_admin(auth_user.id).await.unwrap_or(false);

    let mut sql = "SELECT id, mailbox_id, folder_name, sync_type, status, started_at, finished_at, duration_ms, new_count, updated_count, deleted_count, error_count, error_message FROM sync_runs WHERE 1=1".to_string();

    if let Some(mid) = q.mailbox_id {
        if !is_super && !auth_svc.check_mailbox_permission(auth_user.id, mid, crate::rbac::Permission::View).await.unwrap_or(false) {
            return (StatusCode::FORBIDDEN, Json(json!({ "error": "Access denied for mailbox_id" }))).into_response();
        }
        sql.push_str(" AND mailbox_id = ?");
    } else if !is_super {
        let all_mailboxes: Vec<i64> = sqlx::query_scalar("SELECT id FROM mailboxes WHERE active = 1")
            .fetch_all(&pool)
            .await
            .unwrap_or_default();
        let allowed = auth_svc.filter_viewable_mailboxes(auth_user.id, &all_mailboxes).await.unwrap_or_default();
        if allowed.is_empty() {
            return Json(json!({ "ok": true, "sync_runs": [] })).into_response();
        }
        let in_clause = allowed.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        sql.push_str(&format!(" AND mailbox_id IN ({})", in_clause));
        sql.push_str(" ORDER BY started_at DESC LIMIT ? OFFSET ?");

        let mut query = sqlx::query_as::<_, SyncRunEntry>(&sql);
        for id in allowed {
            query = query.bind(id);
        }
        query = query.bind(limit).bind(offset);
        return match query.fetch_all(&pool).await {
            Ok(runs) => Json(json!({ "ok": true, "sync_runs": runs })).into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "ok": false, "error": e.to_string() }))).into_response(),
        };
    }

    sql.push_str(" ORDER BY started_at DESC LIMIT ? OFFSET ?");
    let mut query = sqlx::query_as::<_, SyncRunEntry>(&sql);
    if let Some(mid) = q.mailbox_id {
        query = query.bind(mid);
    }
    query = query.bind(limit).bind(offset);

    match query.fetch_all(&pool).await {
        Ok(runs) => Json(json!({ "ok": true, "sync_runs": runs })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "ok": false, "error": e.to_string() }))).into_response(),
    }
}

pub fn routes<S>(_pool: &SqlitePool) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    SqlitePool: axum::extract::FromRef<S>,
{
    Router::new()
        .route("/logs", get(list_audit_logs_handler))
        .route("/send", get(list_send_audit_handler))
        .route("/sync-runs", get(list_sync_runs_handler))
}

