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
    pub permissions: Option<Vec<String>>,
    #[serde(default)]
    pub can_view: bool,
    #[serde(default)]
    pub can_read: bool,
    #[serde(default)]
    pub can_reply: bool,
    #[serde(default)]
    pub can_send: bool,
    #[serde(default)]
    pub can_send_as: bool,
    #[serde(default)]
    pub can_mark_read: bool,
    #[serde(default)]
    pub can_move: bool,
    #[serde(default)]
    pub can_delete: bool,
    #[serde(default)]
    pub can_manage: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserReq {
    pub username: String,
    pub password: String,
    pub role: Option<String>,
    pub fallback_email: Option<String>,
    pub domain_ids: Option<Vec<i64>>,
}

#[derive(Debug, Deserialize)]
pub struct SetMailboxCredentialsRequest {
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct UserListItem {
    pub id: i64,
    pub username: Option<String>,
    pub email: Option<String>,
    pub role: String,
    pub fallback_email: Option<String>,
    pub domains: Vec<i64>,
    pub domain_names: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub role: Option<String>,
    pub fallback_email: Option<String>,
    pub domain_ids: Option<Vec<i64>>,
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

    let perms = if let Some(ref list) = req.permissions {
        (
            list.iter().any(|p| p.eq_ignore_ascii_case("view")),
            list.iter().any(|p| p.eq_ignore_ascii_case("read")),
            list.iter().any(|p| p.eq_ignore_ascii_case("reply")),
            list.iter().any(|p| p.eq_ignore_ascii_case("send")),
            list.iter().any(|p| p.eq_ignore_ascii_case("sendas")),
            list.iter().any(|p| p.eq_ignore_ascii_case("markread")),
            list.iter().any(|p| p.eq_ignore_ascii_case("move")),
            list.iter().any(|p| p.eq_ignore_ascii_case("delete")),
            list.iter().any(|p| p.eq_ignore_ascii_case("manage")),
        )
    } else {
        (
            req.can_view,
            req.can_read,
            req.can_reply,
            req.can_send,
            req.can_send_as,
            req.can_mark_read,
            req.can_move,
            req.can_delete,
            req.can_manage,
        )
    };

    match mb_repo.assign_user(req.user_id, mailbox_id, perms, Some(auth_user.id)).await {
        Ok(assign) => (StatusCode::OK, Json(serde_json::json!({ "ok": true, "assignment": assign }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

/// POST /api/v1/rbac/mailboxes/:mailbox_id/credentials
/// SuperAdmin veya DomainAdmin'in posta kutusuna şifre tanımlaması için
async fn set_mailbox_credentials(
    auth_user: AuthUser,
    State(pool): State<SqlitePool>,
    Path(mailbox_id): Path<i64>,
    Json(req): Json<SetMailboxCredentialsRequest>,
) -> impl IntoResponse {
    let auth_svc = AuthorizationService::new(&pool);
    let mb_repo = MailboxRepository::new(&pool);

    // Sadece SuperAdmin veya ilgili domain admin işlem yapabilir
    let is_super = auth_svc.is_super_admin(auth_user.id).await.unwrap_or(false);
    if !is_super {
        let mb = match mb_repo.get_by_id(mailbox_id).await {
            Ok(m) => m,
            Err(_) => return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Posta kutusu bulunamadı" }))).into_response(),
        };
        match auth_svc.can_manage_domain(auth_user.id, mb.domain_id).await {
            Ok(true) => {}
            _ => return (StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": "Bu işlem için yetkiniz yok" }))).into_response(),
        }
    }

    if req.password.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Şifre boş olamaz" }))).into_response();
    }

    // Posta kutusu adresini al
    let mb = match mb_repo.get_by_id(mailbox_id).await {
        Ok(m) => m,
        Err(_) => return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Posta kutusu bulunamadı" }))).into_response(),
    };

    // Şifreyi şifrele ve kaydet
    let encrypted = crate::services::crypto::encrypt_secret(&req.password);
    match mb_repo.save_credentials(mailbox_id, &mb.address, &encrypted).await {
        Ok(_) => {
            // IMAP bağlantısını doğrula (timeout ile)
            let mb_clone = mb.clone();
            let imap_host = {
                // Instance bilgisini domain üzerinden çek
                let imap: Option<String> = sqlx::query_scalar(
                    "SELECT mi.imap_host FROM mailcow_instances mi 
                     JOIN domains d ON d.mailcow_instance_id = mi.id 
                     WHERE d.id = ?"
                )
                .bind(mb_clone.domain_id)
                .fetch_optional(&pool)
                .await
                .unwrap_or(None);
                imap.unwrap_or_default()
            };
            let imap_port: u16 = sqlx::query_scalar::<_, i64>(
                "SELECT mi.imap_port FROM mailcow_instances mi 
                 JOIN domains d ON d.mailcow_instance_id = mi.id 
                 WHERE d.id = ?"
            )
            .bind(mb.domain_id)
            .fetch_optional(&pool)
            .await
            .ok()
            .flatten()
            .unwrap_or(993) as u16;

            let mut verified = false;
            let mut verify_error: Option<String> = None;
            if !imap_host.is_empty() {
                match tokio::time::timeout(
                    std::time::Duration::from_secs(5),
                    crate::imap::conn::connect(&imap_host, imap_port, &mb.address, &req.password),
                ).await {
                    Ok(Ok(mut sess)) => {
                        let _ = sess.session.logout().await;
                        verified = true;
                        // Doğrulama durumunu güncelle
                        let _ = sqlx::query(
                            "UPDATE mailbox_credentials SET verification_status = 'ok', last_verified_at = datetime('now'), verification_error = NULL WHERE mailbox_id = ?"
                        ).bind(mailbox_id).execute(&pool).await;
                    }
                    Ok(Err(e)) => {
                        verify_error = Some(format!("IMAP doğrulama hatası: {}", e));
                        let _ = sqlx::query(
                            "UPDATE mailbox_credentials SET verification_status = 'failed', last_verified_at = datetime('now'), verification_error = ? WHERE mailbox_id = ?"
                        ).bind(verify_error.as_deref()).bind(mailbox_id).execute(&pool).await;
                    }
                    Err(_) => {
                        verify_error = Some("IMAP bağlantı zaman aşımı (5s)".to_string());
                    }
                }
            }

            (StatusCode::OK, Json(serde_json::json!({
                "ok": true,
                "mailbox_id": mailbox_id,
                "address": mb.address,
                "imap_verified": verified,
                "error": verify_error
            }))).into_response()
        },
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

/// GET /api/v1/rbac/domains
async fn list_domains(
    auth_user: AuthUser,
    State(pool): State<SqlitePool>,
) -> impl IntoResponse {
    let auth_svc = AuthorizationService::new(&pool);
    let dom_repo = DomainRepository::new(&pool);

    let is_super = auth_svc.is_super_admin(auth_user.id).await.unwrap_or(false);
    if is_super {
        match dom_repo.list_all().await {
            Ok(list) => (StatusCode::OK, Json(list)).into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
        }
    } else {
        match dom_repo.list_user_domains(auth_user.id).await {
            Ok(list) => (StatusCode::OK, Json(list)).into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
        }
    }
}

/// GET /api/v1/rbac/mailboxes
async fn list_mailboxes(
    auth_user: AuthUser,
    State(pool): State<SqlitePool>,
) -> impl IntoResponse {
    let auth_svc = AuthorizationService::new(&pool);
    let mb_repo = MailboxRepository::new(&pool);
    let dom_repo = DomainRepository::new(&pool);

    let is_super = auth_svc.is_super_admin(auth_user.id).await.unwrap_or(false);
    if is_super {
        match mb_repo.list_all().await {
            Ok(list) => (StatusCode::OK, Json(list)).into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
        }
    } else {
        let user_domains = match dom_repo.list_user_domains(auth_user.id).await {
            Ok(list) => list,
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
        };

        let mut all_mbs = Vec::new();
        for dom in user_domains {
            if let Ok(mbs) = mb_repo.list_by_domain(dom.id).await {
                all_mbs.extend(mbs);
            }
        }
        (StatusCode::OK, Json(all_mbs)).into_response()
    }
}

/// GET /api/v1/rbac/users
async fn list_users(
    auth_user: AuthUser,
    State(pool): State<SqlitePool>,
) -> impl IntoResponse {
    let auth_svc = AuthorizationService::new(&pool);
    let is_super = auth_svc.is_super_admin(auth_user.id).await.unwrap_or(false);

    let user_repo = UserRepository::new(&pool);
    let dom_repo = DomainRepository::new(&pool);

    // Get domains managed by DomainAdmin (if not super admin)
    let managed_domain_ids = if !is_super {
        let doms = dom_repo.list_user_domains(auth_user.id).await.unwrap_or_default();
        doms.into_iter().map(|d| d.id).collect::<Vec<i64>>()
    } else {
        Vec::new()
    };

    let all_users = match user_repo.list_all().await {
        Ok(users) => users,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    };

    let mut result = Vec::new();

    for u in all_users {
        // Fetch explicit domains for this user
        let mut domains: Vec<i64> = sqlx::query_scalar(
            "SELECT domain_id FROM user_domain_assignments WHERE user_id = ?"
        )
        .bind(u.id)
        .fetch_all(&pool)
        .await
        .unwrap_or_default();

        // Also fetch from domain_admin_assignments if DomainAdmin
        if u.role == "DomainAdmin" {
            let admin_doms: Vec<i64> = sqlx::query_scalar(
                "SELECT domain_id FROM domain_admin_assignments WHERE user_id = ? AND revoked_at IS NULL"
            )
            .bind(u.id)
            .fetch_all(&pool)
            .await
            .unwrap_or_default();
            domains.extend(admin_doms);
        }

        // Implicit domains (based on email domain match)
        if let Some(ref email) = u.email {
            if let Some(domain_part) = email.split('@').nth(1) {
                let implicit_dom_id: Option<i64> = sqlx::query_scalar(
                    "SELECT id FROM domains WHERE name = ? AND deleted_at IS NULL"
                )
                .bind(domain_part)
                .fetch_optional(&pool)
                .await
                .unwrap_or(None);
                if let Some(did) = implicit_dom_id {
                    if !domains.contains(&did) {
                        domains.push(did);
                    }
                }
            }
        }

        // Get domain names
        let mut domain_names = Vec::new();
        for &did in &domains {
            if let Ok(dom) = dom_repo.get_by_id(did).await {
                domain_names.push(dom.name);
            }
        }

        let is_visible = is_super || u.id == auth_user.id || domains.iter().any(|d| managed_domain_ids.contains(d));

        if is_visible {
            result.push(UserListItem {
                id: u.id,
                username: u.username,
                email: u.email,
                role: u.role,
                fallback_email: u.fallback_email,
                domains,
                domain_names,
            });
        }
    }

    (StatusCode::OK, Json(result)).into_response()
}

/// POST /api/v1/rbac/users
async fn create_user(
    auth_user: AuthUser,
    State(pool): State<SqlitePool>,
    Json(req): Json<CreateUserReq>,
) -> impl IntoResponse {
    let auth_svc = AuthorizationService::new(&pool);
    let is_super = auth_svc.is_super_admin(auth_user.id).await.unwrap_or(false);

    if req.username.trim().is_empty() || req.password.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Kullanıcı adı ve şifre zorunludur" }))).into_response();
    }

    let role = req.role.unwrap_or_else(|| "User".to_string());
    if role == "SuperAdmin" && !is_super {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": "Sadece SuperAdmin yetki verebilir" }))).into_response();
    }

    let hash_str = match bcrypt::hash(&req.password, bcrypt::DEFAULT_COST) {
        Ok(h) => h,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    };

    let user_id = match sqlx::query_scalar::<_, i64>(
        "INSERT INTO users (username, email, password_hash, role, fallback_email, created_at, updated_at) 
         VALUES (?, ?, ?, ?, ?, datetime('now'), datetime('now')) 
         RETURNING id"
    )
    .bind(&req.username)
    .bind(&req.username)
    .bind(&hash_str)
    .bind(&role)
    .bind(req.fallback_email.as_deref())
    .fetch_one(&pool)
    .await {
        Ok(id) => id,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": format!("Kullanıcı eklenemedi: {}", e) }))).into_response(),
    };

    // Auto assign domain groups if provided or matching email
    if let Some(domain_ids) = req.domain_ids {
        for did in domain_ids {
            if role == "DomainAdmin" {
                let dom_repo = DomainRepository::new(&pool);
                let _ = dom_repo.assign_admin(user_id, did, Some(auth_user.id)).await;
            } else {
                let _ = sqlx::query(
                    "INSERT OR IGNORE INTO user_domain_assignments (user_id, domain_id, created_at) VALUES (?, ?, datetime('now'))"
                )
                .bind(user_id)
                .bind(did)
                .execute(&pool)
                .await;
            }
        }
    } else {
        crate::services::auth_service::auto_assign_user_domain(&pool, user_id, &req.username).await;
    }

    (StatusCode::OK, Json(serde_json::json!({ "ok": true, "id": user_id }))).into_response()
}

/// POST /api/v1/rbac/users/:user_id
async fn update_user_details(
    auth_user: AuthUser,
    State(pool): State<SqlitePool>,
    Path(target_user_id): Path<i64>,
    Json(req): Json<UpdateUserRequest>,
) -> impl IntoResponse {
    let auth_svc = AuthorizationService::new(&pool);
    let is_super = auth_svc.is_super_admin(auth_user.id).await.unwrap_or(false);

    let dom_repo = DomainRepository::new(&pool);
    let user_repo = UserRepository::new(&pool);

    // 1. Get managed domains if DomainAdmin
    let managed_domain_ids = if !is_super {
        let doms = dom_repo.list_user_domains(auth_user.id).await.unwrap_or_default();
        doms.into_iter().map(|d| d.id).collect::<Vec<i64>>()
    } else {
        Vec::new()
    };

    // 2. Fetch target user
    let target_user = match user_repo.get_by_id(target_user_id).await {
        Ok(u) => u,
        Err(_) => return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Kullanıcı bulunamadı" }))).into_response(),
    };

    // 3. Authorization check
    if !is_super {
        if target_user.id != auth_user.id {
            let mut target_domains: Vec<i64> = sqlx::query_scalar(
                "SELECT domain_id FROM user_domain_assignments WHERE user_id = ?"
            )
            .bind(target_user_id)
            .fetch_all(&pool)
            .await
            .unwrap_or_default();

            if target_user.role == "DomainAdmin" {
                let target_admin_doms: Vec<i64> = sqlx::query_scalar(
                    "SELECT domain_id FROM domain_admin_assignments WHERE user_id = ? AND revoked_at IS NULL"
                )
                .bind(target_user_id)
                .fetch_all(&pool)
                .await
                .unwrap_or_default();
                target_domains.extend(target_admin_doms);
            }

            if let Some(ref email) = target_user.email {
                if let Some(domain_part) = email.split('@').nth(1) {
                    if let Ok(Some(did)) = sqlx::query_scalar::<_, Option<i64>>("SELECT id FROM domains WHERE name = ? AND deleted_at IS NULL").bind(domain_part).fetch_optional(&pool).await {
                        if let Some(did) = did {
                            target_domains.push(did);
                        }
                    }
                }
            }

            let matches = target_domains.iter().any(|d| managed_domain_ids.contains(d));
            if !matches {
                return (StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": "Bu kullanıcıyı düzenleme yetkiniz yok" }))).into_response();
            }
        }
    }

    // 4. Update role and fallback email
    if let Some(role) = req.role {
        if role == "SuperAdmin" && !is_super {
            return (StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": "Sadece SuperAdmin yetki verebilir" }))).into_response();
        }
        let _ = user_repo.update_role(target_user_id, &role).await;
    }

    if let Some(fallback) = req.fallback_email {
        let _ = sqlx::query("UPDATE users SET fallback_email = ?, updated_at = datetime('now') WHERE id = ?")
            .bind(&fallback)
            .bind(target_user_id)
            .execute(&pool)
            .await;
    }

    // 5. Update domain assignments
    if let Some(domain_ids) = req.domain_ids {
        if !is_super {
            for &did in &domain_ids {
                if !managed_domain_ids.contains(&did) {
                    return (StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": "Sadece yönettiğiniz domain gruplarına atama yapabilirsiniz" }))).into_response();
                }
            }
        }

        let current_user = user_repo.get_by_id(target_user_id).await.unwrap_or(target_user);

        if current_user.role == "DomainAdmin" {
            if is_super {
                let _ = sqlx::query("DELETE FROM domain_admin_assignments WHERE user_id = ?")
                    .bind(target_user_id)
                    .execute(&pool)
                    .await;
            } else {
                let query_str = format!(
                    "DELETE FROM domain_admin_assignments WHERE user_id = ? AND domain_id IN ({})",
                    managed_domain_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",")
                );
                let mut q = sqlx::query(&query_str).bind(target_user_id);
                for &did in &managed_domain_ids {
                    q = q.bind(did);
                }
                let _ = q.execute(&pool).await;
            }

            for did in domain_ids {
                let _ = dom_repo.assign_admin(target_user_id, did, Some(auth_user.id)).await;
            }
        } else {
            if is_super {
                let _ = sqlx::query("DELETE FROM user_domain_assignments WHERE user_id = ?")
                    .bind(target_user_id)
                    .execute(&pool)
                    .await;
            } else {
                let query_str = format!(
                    "DELETE FROM user_domain_assignments WHERE user_id = ? AND domain_id IN ({})",
                    managed_domain_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",")
                );
                let mut q = sqlx::query(&query_str).bind(target_user_id);
                for &did in &managed_domain_ids {
                    q = q.bind(did);
                }
                let _ = q.execute(&pool).await;
            }

            for did in domain_ids {
                let _ = sqlx::query(
                    "INSERT OR IGNORE INTO user_domain_assignments (user_id, domain_id, created_at) VALUES (?, ?, datetime('now'))"
                )
                .bind(target_user_id)
                .bind(did)
                .execute(&pool)
                .await;
            }
        }
    }

    let updated = user_repo.get_by_id(target_user_id).await.unwrap();
    (StatusCode::OK, Json(serde_json::json!({ "ok": true, "user": updated }))).into_response()
}

pub fn routes<S>(pool: &SqlitePool) -> Router<S>
where
    S: Send + Sync + Clone + 'static,
    SqlitePool: axum::extract::FromRef<S>,
{
    let _ = pool;
    Router::new()
        .route("/me/permissions", get(get_my_permissions))
        .route("/users", get(list_users).post(create_user))
        .route("/users/:user_id", post(update_user_details))
        .route("/users/:user_id/role", post(update_user_role))
        .route("/domains", get(list_domains))
        .route("/domains/:domain_id/admins/:user_id", post(assign_domain_admin).delete(revoke_domain_admin))
        .route("/mailboxes", get(list_mailboxes))
        .route("/mailboxes/:mailbox_id/assignments", post(assign_mailbox_user))
        .route("/mailboxes/:mailbox_id/credentials", post(set_mailbox_credentials))
}
