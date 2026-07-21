use anyhow::Result;
use sqlx::SqlitePool;
use crate::domains::DomainRepository;
use crate::mailboxes::MailboxRepository;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permission {
    View,
    Read,
    Reply,
    Send,
    SendAs,
    MarkRead,
    Move,
    Delete,
    Manage,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MailboxPermissions {
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

impl MailboxPermissions {
    pub fn all_true() -> Self {
        Self {
            can_view: true,
            can_read: true,
            can_reply: true,
            can_send: true,
            can_send_as: true,
            can_mark_read: true,
            can_move: true,
            can_delete: true,
            can_manage: true,
        }
    }

    pub fn all_false() -> Self {
        Self {
            can_view: false,
            can_read: false,
            can_reply: false,
            can_send: false,
            can_send_as: false,
            can_mark_read: false,
            can_move: false,
            can_delete: false,
            can_manage: false,
        }
    }

    pub fn has_permission(&self, perm: Permission) -> bool {
        match perm {
            Permission::View => self.can_view,
            Permission::Read => self.can_read,
            Permission::Reply => self.can_reply,
            Permission::Send => self.can_send,
            Permission::SendAs => self.can_send_as,
            Permission::MarkRead => self.can_mark_read,
            Permission::Move => self.can_move,
            Permission::Delete => self.can_delete,
            Permission::Manage => self.can_manage,
        }
    }
}

pub struct AuthorizationService<'a> {
    pub pool: &'a SqlitePool,
}

impl<'a> AuthorizationService<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    /// Check if user is SuperAdmin
    pub async fn is_super_admin(&self, user_id: i64) -> Result<bool> {
        let role: Option<String> = sqlx::query_scalar("SELECT role FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_optional(self.pool)
            .await?;
        Ok(role.as_deref() == Some("SuperAdmin"))
    }

    /// Check if user is SuperAdmin or DomainAdmin for the given domain
    pub async fn can_manage_domain(&self, user_id: i64, domain_id: i64) -> Result<bool> {
        if self.is_super_admin(user_id).await? {
            return Ok(true);
        }
        let dom_repo = DomainRepository::new(self.pool);
        let domains = dom_repo.list_user_domains(user_id).await?;
        Ok(domains.iter().any(|d| d.id == domain_id))
    }

    /// Get user's effective permissions for a specific mailbox
    pub async fn get_mailbox_permissions(
        &self,
        user_id: i64,
        mailbox_id: i64,
    ) -> Result<MailboxPermissions> {
        if self.is_super_admin(user_id).await? {
            return Ok(MailboxPermissions::all_true());
        }

        let mb_repo = MailboxRepository::new(self.pool);
        let mb = match mb_repo.get_by_id(mailbox_id).await {
            Ok(m) => m,
            Err(_) => return Ok(MailboxPermissions::all_false()),
        };

        if self.can_manage_domain(user_id, mb.domain_id).await? {
            return Ok(MailboxPermissions::all_true());
        }

        let assigned = mb_repo.list_user_mailboxes(user_id).await?;
        for (m, assign) in assigned {
            if m.id == mailbox_id {
                return Ok(MailboxPermissions {
                    can_view: assign.can_view,
                    can_read: assign.can_read,
                    can_reply: assign.can_reply,
                    can_send: assign.can_send,
                    can_send_as: assign.can_send_as,
                    can_mark_read: assign.can_mark_read,
                    can_move: assign.can_move,
                    can_delete: assign.can_delete,
                    can_manage: assign.can_manage,
                });
            }
        }

        Ok(MailboxPermissions::all_false())
    }

    /// Check if user has a specific permission on a mailbox
    pub async fn check_mailbox_permission(
        &self,
        user_id: i64,
        mailbox_id: i64,
        perm: Permission,
    ) -> Result<bool> {
        let perms = self.get_mailbox_permissions(user_id, mailbox_id).await?;
        Ok(perms.has_permission(perm))
    }

    /// Filter a list of mailbox IDs down to only those where the user has at least View access
    pub async fn filter_viewable_mailboxes(
        &self,
        user_id: i64,
        mailbox_ids: &[i64],
    ) -> Result<Vec<i64>> {
        if self.is_super_admin(user_id).await? {
            return Ok(mailbox_ids.to_vec());
        }
        let mut allowed = Vec::new();
        for &id in mailbox_ids {
            if self
                .check_mailbox_permission(user_id, id, Permission::View)
                .await?
            {
                allowed.push(id);
            }
        }
        Ok(allowed)
    }

    /// Return all mailbox IDs that this user can view
    pub async fn get_allowed_mailbox_ids(&self, user_id: i64) -> Result<Vec<i64>> {
        if self.is_super_admin(user_id).await? {
            let ids: Vec<i64> = sqlx::query_scalar(
                "SELECT id FROM mailboxes WHERE deleted_at IS NULL AND active = 1",
            )
            .fetch_all(self.pool)
            .await?;
            return Ok(ids);
        }

        let mut allowed = std::collections::HashSet::new();

        let dom_repo = DomainRepository::new(self.pool);
        let domains = dom_repo.list_user_domains(user_id).await?;
        let mb_repo = MailboxRepository::new(self.pool);
        for d in domains {
            let mbs = mb_repo.list_by_domain(d.id).await?;
            for mb in mbs {
                allowed.insert(mb.id);
            }
        }

        let assigned = mb_repo.list_user_mailboxes(user_id).await?;
        for (mb, assign) in assigned {
            if assign.can_view {
                allowed.insert(mb.id);
            }
        }

        let mut sorted: Vec<i64> = allowed.into_iter().collect();
        sorted.sort_unstable();
        Ok(sorted)
    }
}
