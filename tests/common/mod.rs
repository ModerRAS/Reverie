#![allow(dead_code)]
//! Common E2E test infrastructure for Reverie
//!
//! Provides `TestContext` for starting/stopping a full Reverie server
//! and making API requests. Uses an in-memory database so tests are
//! isolated and can run in parallel.
//!
//! ## Usage
//!
//! ```rust,ignore
//! #[path = "../common/mod.rs"]
//! mod common;
//! use common::*;
//!
//! #[tokio::test]
//! async fn test_example() {
//!     let ctx = TestContext::new().await.unwrap();
//!     let resp = ctx.subsonic_get("ping", &[("f", "json")]).await.unwrap();
//!     assert_eq!(resp["subsonic-response"]["status"], "ok");
//! }
//! ```

use std::sync::Arc;
use std::thread;
use std::time::Duration;

use reverie_core::Track;
use reverie_server::{run_with_storage, ServerRunConfig};
use reverie_storage::{DatabaseConfig, DatabaseStorage, Storage, TrackStorage};

// ---------------------------------------------------------------------------
// Global singleton — we start the server ONCE and reuse it across all tests
// in the same test binary process.
// Uses tokio::sync::OnceCell for async init (avoids the "block_on inside
// runtime" panic that std::sync::OnceLock would cause).
// The server itself runs in a dedicated OS thread with its own tokio runtime
// so it survives across #[tokio::test] runtime boundaries.
// ---------------------------------------------------------------------------
static TEST_SERVER: tokio::sync::OnceCell<TestServerHandle> = tokio::sync::OnceCell::const_new();

/// Obtain (or start) the shared test server.
async fn get_or_init_server() -> &'static TestServerHandle {
    TEST_SERVER
        .get_or_init(|| async {
            TestServerHandle::start()
                .await
                .expect("Failed to start test server")
        })
        .await
}

/// Handle to a running test server instance.
///
/// Created once per test binary via `TestContext::new()` and reused.
/// The server runs in a dedicated OS thread with its own tokio runtime,
/// so it survives between #[tokio::test] runtime lifecycles.
pub struct TestServerHandle {
    pub base_url: String,
    pub storage: Arc<DatabaseStorage>,
    /// Signal graceful shutdown to the server
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
    /// Dedicated OS thread running the server
    _server_thread: Option<thread::JoinHandle<()>>,
}

impl TestServerHandle {
    /// Start the reverie server on a random port with an in-memory database.
    ///
    /// The server runs in a dedicated OS thread with its own tokio runtime
    /// so it persists across `#[tokio::test]` runtime boundaries.
    pub async fn start() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let port = portpicker::pick_unused_port().ok_or("No available port")?;
        let base_url = format!("http://127.0.0.1:{}", port);

        // In-memory SQLite — each test binary gets a fresh database.
        let config = DatabaseConfig::memory();
        let storage = Arc::new(DatabaseStorage::new(config).await?);
        storage.initialize().await?;

        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

        let server_config = ServerRunConfig {
            host: "127.0.0.1".to_string(),
            port,
            enable_cors: true,
            ui_dir: None,
            shutdown_rx: Some(shutdown_rx),
            ..Default::default()
        };

        let storage_for_server = storage.clone();

        // Spawn the server in a dedicated OS thread with its own tokio runtime.
        // This ensures the server survives across #[tokio::test] runtime boundaries —
        // when a test's runtime is dropped, the server thread keeps running.
        let server_thread = thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("Failed to create server runtime");
            rt.block_on(async move {
                if let Err(e) = run_with_storage(storage_for_server, server_config).await {
                    tracing::error!("Test server error: {e}");
                }
            });
        });

        // Poll until the server answers.
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()?;

        for _retry in 0..30 {
            if client
                .get(format!("{}/rest/ping", base_url))
                .query(&[("f", "json")])
                .send()
                .await
                .is_ok()
            {
                return Ok(Self {
                    base_url,
                    storage,
                    shutdown_tx: Some(shutdown_tx),
                    _server_thread: Some(server_thread),
                });
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }

        Err("Server failed to start within 6-second timeout".into())
    }
}

impl Drop for TestServerHandle {
    fn drop(&mut self) {
        // Send shutdown signal — the server thread will receive it via
        // shutdown_rx after server.start() naturally stops (which only
        // happens on process exit or error).
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        // We intentionally do NOT join the server thread here.
        // run_with_storage blocks in server.start() (axum::serve) forever,
        // so joining would hang. The thread will be cleaned up on process exit.
    }
}

// ---------------------------------------------------------------------------
// TestContext — convenient wrapper for test cases
// ---------------------------------------------------------------------------

/// E2E test context.
///
/// Wraps the singleton server handle and an HTTP client so tests can
/// easily issue API requests and seed data.
pub struct TestContext {
    pub server: &'static TestServerHandle,
    pub client: reqwest::Client,
}

impl TestContext {
    /// Create a new test context.
    ///
    /// Starts the server on first call (uses per-process `std::sync::OnceLock`)
    /// and reuses it for subsequent calls within the same test binary.
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let server = get_or_init_server().await;
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;
        Ok(Self { server, client })
    }

    /// Base URL of the running server (e.g. `http://127.0.0.1:51234`).
    pub fn base_url(&self) -> &str {
        &self.server.base_url
    }

    /// Direct access to the underlying `DatabaseStorage` for seeding data.
    pub fn storage(&self) -> &Arc<DatabaseStorage> {
        &self.server.storage
    }

    // -- Subsonic API helpers ------------------------------------------------

    /// Build a Subsonic API URL with query parameters.
    pub fn subsonic_url(&self, endpoint: &str, params: &[(&str, &str)]) -> String {
        let mut url = format!("{}/rest/{}", self.base_url(), endpoint);
        if !params.is_empty() {
            url.push('?');
            for (i, (k, v)) in params.iter().enumerate() {
                if i > 0 {
                    url.push('&');
                }
                url.push_str(&format!("{}={}", k, urlencoding::encode(v)));
            }
        }
        url
    }

    /// Send a GET request to a Subsonic endpoint and parse the JSON response.
    pub async fn subsonic_get(
        &self,
        endpoint: &str,
        params: &[(&str, &str)],
    ) -> Result<serde_json::Value, reqwest::Error> {
        let url = self.subsonic_url(endpoint, params);
        self.client.get(&url).send().await?.json().await
    }

    // -- Data seeding helpers ------------------------------------------------

    /// Insert a single track into the database.
    pub async fn seed_track(
        &self,
        track: Track,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.storage().save_track(&track).await?;
        Ok(())
    }

    /// Insert multiple tracks into the database.
    pub async fn seed_tracks(
        &self,
        tracks: Vec<Track>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let storage = self.storage();
        for track in &tracks {
            storage.save_track(track).await?;
        }
        Ok(())
    }
}
