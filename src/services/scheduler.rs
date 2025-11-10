use std::time::Duration;
use anyhow::Result;
use sqlx::SqlitePool;
use tracing::{info, warn};

use crate::services::{account_service, message_sync_service};

/// Starts a lightweight sync scheduler. Every tick it iterates accounts and runs a delta sync.
pub fn start(pool: SqlitePool) {
    tokio::spawn(async move {
        loop {
            // Safety tick: run every 60s; each account has its own sync_frequency_secs to throttle inside loop
            let tick_start = std::time::Instant::now();
            match account_service::list_accounts(&pool).await {
                Ok(accounts) => {
                    for acc in accounts {
                        if !acc.enabled { continue; }
                        // Skip too frequent syncs: compare last_sync_ts with sync_frequency_secs
                        if let Some(last) = acc.last_sync_ts {
                            let now = chrono::Utc::now().timestamp();
                            if now - last < acc.sync_frequency_secs { continue; }
                        }
                        let p = pool.clone();
                        tokio::spawn(async move {
                            match message_sync_service::sync_account_messages(&p, &acc).await {
                                Ok(stats) => {
                                    info!(email=%acc.email, folders=%stats.len(), "scheduled sync completed");
                                    let _ = crate::services::account_service::update_last_sync(&p, &acc.id).await;
                                }
                                Err(e) => warn!(email=%acc.email, error=%e.to_string(), "scheduled sync failed"),
                            }
                        });
                    }
                }
                Err(e) => warn!("scheduler: list_accounts failed: {}", e),
            }
            // Body cache GC
            crate::services::message_body_service::gc(&pool, 5000).await;
            // sleep remaining out of 60s
            let elapsed = tick_start.elapsed();
            let sleep_ms = 60_000u64.saturating_sub(elapsed.as_millis() as u64);
            tokio::time::sleep(Duration::from_millis(sleep_ms.max(1))).await;
        }
    });
}
