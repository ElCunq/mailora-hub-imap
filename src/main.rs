use anyhow::Result;
use axum::{routing::get, Router};
use std::sync::Arc;
use tower_http::services::ServeDir;
use tracing_subscriber::EnvFilter;

mod db;
mod imap;
mod models;
mod oauth;
mod persist;
mod routes;
mod services;
mod smtp;

#[derive(Clone)]
struct AppState {
    pool: sqlx::SqlitePool,
    idle_manager: Arc<services::idle_watcher_service::IdleWatcherManager>,
    oauth_manager: Arc<oauth::OAuthManager>,
}

impl axum::extract::FromRef<AppState> for sqlx::SqlitePool {
    fn from_ref(state: &AppState) -> Self {
        state.pool.clone()
    }
}

impl axum::extract::FromRef<AppState> for Arc<services::idle_watcher_service::IdleWatcherManager> {
    fn from_ref(state: &AppState) -> Self {
        state.idle_manager.clone()
    }
}

impl axum::extract::FromRef<AppState> for Arc<oauth::OAuthManager> {
    fn from_ref(state: &AppState) -> Self {
        state.oauth_manager.clone()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    // load persisted accounts
    if let Err(e) = persist::load_state().await {
        tracing::warn!("persist load error: {e}");
    }

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,mailora_hub_imap=debug")),
        )
        .init();

    // Build a correct sqlite URL (sqlx expects sqlite://path or sqlite::memory:)
    let raw_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://mailora_imap.db".into());
    let db_url = normalize_sqlite_url(&raw_url);

    if std::path::Path::new("migrations").exists() {
        // Ensure file exists for file-based sqlite (avoid open error on some setups)
        if let Some(path) = db_file_path(&db_url) {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            if !path.exists() {
                std::fs::File::create(&path).ok();
            }
        }
        let pool = sqlx::SqlitePool::connect(&db_url).await?;
        if let Err(e) = db::run_migrations(&pool).await {
            tracing::warn!("migration error: {e}");
        }
        if let Err(e) = db::seed_account(&pool).await {
            tracing::warn!("seed error: {e}");
        }

        // Create idle watcher manager
        let idle_manager = Arc::new(services::idle_watcher_service::IdleWatcherManager::new());

        // Create OAuth manager
        let oauth_manager = Arc::new(oauth::OAuthManager::new());

        let state = AppState {
            pool: pool.clone(),
            idle_manager: idle_manager.clone(),
            oauth_manager: oauth_manager.clone(),
        };

        let idle_routes = Router::new()
            .route(
                "/idle/start/:account_id",
                axum::routing::post(routes::idle::start_idle_watcher),
            )
            .route(
                "/idle/stop/:account_id",
                axum::routing::post(routes::idle::stop_idle_watcher),
            )
            .route("/idle/status", get(routes::idle::idle_status))
            .route("/idle/events", get(routes::idle::idle_events_stream));

        let oauth_routes = Router::new()
            .route("/oauth/start", get(routes::oauth::start_oauth))
            .route("/oauth/callback", get(routes::oauth::oauth_callback))
            .route("/oauth/setup-guide", get(routes::oauth::oauth_setup_guide))
            .with_state(oauth_manager.clone());

        let app = Router::new()
            .route("/healthz", get(|| async { "ok" }))
            .merge(routes::routes())
            .merge(idle_routes)
            .merge(oauth_routes)
            .nest_service("/static", ServeDir::new("static"))
            // App state
            .with_state(state.clone());

        let port: u16 = std::env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(3030); // Stalwart 8787'de, biz 3030'da çalışalım

        let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
        tracing::info!("listening on http://{}", addr);
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal())
            .await?;
    } else {
        tracing::warn!("migrations folder not found, skipping DB setup");
    }
    // save state after shutdown
    if let Err(e) = persist::save_state().await {
        tracing::warn!("persist save error: {e}");
    }
    Ok(())
}

async fn shutdown_signal() {
    use tokio::signal;
    let ctrl_c = async {
        signal::ctrl_c().await.ok();
    };
    #[cfg(unix)]
    let term = async {
        if let Ok(mut s) = signal::unix::signal(signal::unix::SignalKind::terminate()) {
            s.recv().await;
        }
    };
    #[cfg(not(unix))]
    let term = std::future::pending::<()>();
    tokio::select! { _ = ctrl_c => {}, _ = term => {} }
}

fn normalize_sqlite_url(input: &str) -> String {
    // Accept forms: sqlite:foo.db (fix), sqlite://foo.db (ok), file:foo.db (convert), just path (prepend)
    if input.starts_with("sqlite://") || input.starts_with("sqlite::memory:") {
        return input.to_string();
    }
    if input.starts_with("sqlite:") {
        // single colon like sqlite:foo.db -> make it sqlite://foo.db
        let rest = input.trim_start_matches("sqlite:");
        return format!("sqlite://{}", rest.trim_start_matches('/'));
    }
    if input.starts_with("file:") {
        return format!("sqlite://{}", input.trim_start_matches("file:"));
    }
    // bare path
    format!("sqlite://{}", input)
}

fn db_file_path(url: &str) -> Option<std::path::PathBuf> {
    // sqlite URLs: sqlite://<path>. Strip prefix
    if let Some(rest) = url.strip_prefix("sqlite://") {
        if rest == ":memory:" {
            return None;
        }
        return Some(std::path::PathBuf::from(rest));
    }
    None
}

// index.html serve için routes/mod.rs içinde root_page kullanılıyor.
