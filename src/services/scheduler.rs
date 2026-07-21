use std::time::Duration;
use sqlx::SqlitePool;
use tracing::{info, warn};




/// Starts a lightweight sync scheduler. Every tick it iterates accounts and runs a delta sync.
pub fn start(pool: SqlitePool) {
    let discovery_pool = pool.clone();
    tokio::spawn(async move {
        let service = crate::mailcow::discovery::DiscoveryService::new(&discovery_pool);
        loop {
            match service.run_discovery_all().await {
                Ok(summaries) => {
                    if !summaries.is_empty() {
                        info!(instances = summaries.len(), "Periodic Mailcow discovery completed successfully");
                    }
                }
                Err(e) => {
                    warn!(error = %e.to_string(), "Periodic Mailcow discovery encountered error");
                }
            }
            tokio::time::sleep(Duration::from_secs(30)).await;
        }
    });
}
