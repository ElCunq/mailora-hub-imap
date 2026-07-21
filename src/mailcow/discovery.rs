use anyhow::Result;
use chrono::Utc;
use sqlx::SqlitePool;
use crate::domains::model::UpsertDomain;
use crate::domains::DomainRepository;
use crate::mailboxes::model::UpsertMailbox;
use crate::mailboxes::MailboxRepository;
use crate::mailcow::client::MailcowClient;
use crate::mailcow::MailcowRepository;

pub struct DiscoveryService<'a> {
    pub pool: &'a SqlitePool,
}

#[derive(Debug, Clone, serde::Serialize, PartialEq)]
pub struct DiscoverySummary {
    pub instance_id: i64,
    pub instance_name: String,
    pub domains_upserted: usize,
    pub domains_soft_deleted: u64,
    pub mailboxes_upserted: usize,
    pub mailboxes_soft_deleted: u64,
}

impl<'a> DiscoveryService<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn run_discovery_for_instance(&self, instance_id: i64) -> Result<DiscoverySummary> {
        let mc_repo = MailcowRepository::new(self.pool);
        let instance = mc_repo.get_by_id(instance_id).await?;
        if !instance.enabled {
            anyhow::bail!("Mailcow instance {} is disabled", instance.name);
        }

        let client = MailcowClient::from_instance(&instance);
        let start_time_str = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        // 1. Fetch & upsert domains
        let domains = match client.fetch_domains().await {
            Ok(d) => d,
            Err(e) => {
                let _ = mc_repo
                    .update_discovery_status(instance.id, "error", Some(&e.to_string()))
                    .await;
                return Err(e);
            }
        };

        let domain_repo = DomainRepository::new(self.pool);
        let mailbox_repo = MailboxRepository::new(self.pool);

        let mut domains_upserted = 0;
        for d in &domains {
            domain_repo
                .upsert(UpsertDomain {
                    mailcow_instance_id: instance.id,
                    external_id: None,
                    name: d.name.clone(),
                    active: d.active,
                    quota: d.quota,
                    mailbox_limit: d.mailbox_limit,
                })
                .await?;
            domains_upserted += 1;
        }

        let domains_soft_deleted = domain_repo
            .soft_delete_unseen(instance.id, &start_time_str)
            .await?;

        // 2. Fetch & upsert mailboxes
        let mailboxes = match client.fetch_mailboxes().await {
            Ok(m) => m,
            Err(e) => {
                let _ = mc_repo
                    .update_discovery_status(instance.id, "error", Some(&e.to_string()))
                    .await;
                return Err(e);
            }
        };

        let active_domains = domain_repo.list_by_instance(instance.id).await?;
        let mut mailboxes_upserted = 0;
        let mut total_mailboxes_soft_deleted = 0;

        for m in &mailboxes {
            let domain_part = match m.address.split('@').nth(1) {
                Some(dp) => dp,
                None => continue,
            };

            let matched_domain = active_domains
                .iter()
                .find(|dom| dom.name.eq_ignore_ascii_case(domain_part));
            if let Some(dom) = matched_domain {
                mailbox_repo
                    .upsert(UpsertMailbox {
                        domain_id: dom.id,
                        external_id: None,
                        address: m.address.clone(),
                        local_part: m.local_part.clone(),
                        display_name: m.display_name.clone(),
                        active: m.active,
                        quota: m.quota,
                        used_quota: m.used_quota,
                    })
                    .await?;
                mailboxes_upserted += 1;
            }
        }

        for dom in &active_domains {
            let deleted = mailbox_repo
                .soft_delete_unseen(dom.id, &start_time_str)
                .await?;
            total_mailboxes_soft_deleted += deleted;
        }

        mc_repo
            .update_discovery_status(instance.id, "success", None)
            .await?;

        Ok(DiscoverySummary {
            instance_id: instance.id,
            instance_name: instance.name,
            domains_upserted,
            domains_soft_deleted,
            mailboxes_upserted,
            mailboxes_soft_deleted: total_mailboxes_soft_deleted,
        })
    }

    pub async fn run_discovery_all(&self) -> Result<Vec<DiscoverySummary>> {
        let mc_repo = MailcowRepository::new(self.pool);
        let instances = mc_repo.list_all().await?;
        let mut summaries = Vec::new();

        for inst in instances {
            if !inst.enabled {
                continue;
            }
            if let Ok(sum) = self.run_discovery_for_instance(inst.id).await {
                summaries.push(sum);
            }
        }

        Ok(summaries)
    }
}
