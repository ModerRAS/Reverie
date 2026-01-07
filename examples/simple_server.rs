//! Example: Using Reverie with in-memory storage
//!
//! This example demonstrates how to:
//! 1. Initialize storage
//! 2. Add sample data
//! 3. Start the HTTP server
//! 4. Query the API
//!
//! Run with: cargo run --example simple_server

use anyhow::Result;
use chrono::Utc;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

use reverie_core::{Album, Artist, Track};
use reverie_network::{axum_server::AxumServer, HttpServer, NetworkConfig};
use reverie_storage::{memory::MemoryStorage, AlbumStorage, ArtistStorage, Storage, TrackStorage};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Reverie Example: Simple Server with Sample Data");

    // Initialize storage
    let storage = Arc::new(MemoryStorage::new());
    storage.initialize().await?;

    // Add sample data
    add_sample_data(&storage).await?;

    // Create network configuration
    let network_config = NetworkConfig {
        host: "127.0.0.1".to_string(),
        port: 4533,
        enable_cors: true,
        max_body_size: 10 * 1024 * 1024,
        timeout_seconds: 30,
    };

    // Create and start HTTP server
    let server = AxumServer::new(storage.clone(), network_config.clone());
    let addr = format!("{}:{}", network_config.host, network_config.port)
        .parse()
        .expect("Invalid server address");

    tracing::info!("Starting server on {}", addr);
    tracing::info!("Try:");
    tracing::info!("  curl http://127.0.0.1:4533/health");
    tracing::info!("  curl http://127.0.0.1:4533/api/tracks");
    tracing::info!("  curl http://127.0.0.1:4533/api/albums");
    tracing::info!("  curl http://127.0.0.1:4533/api/artists");

    server.start(addr).await?;

    Ok(())
}

async fn add_sample_data(storage: &MemoryStorage) -> Result<()> {
    tracing::info!("Adding sample data...");

    // Create sample artist
    let artist = Artist {
        id: Uuid::new_v4(),
        name: "The Example Band".to_string(),
        bio: Some("A fictional band for demonstration purposes.".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    storage.save_artist(&artist).await?;

    // Create sample album
    let album = Album {
        id: Uuid::new_v4(),
        name: "Greatest Hits".to_string(),
        artist_id: Some(artist.id),
        year: Some(2024),
        genre: Some("Demo".to_string()),
        cover_art_path: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    storage.save_album(&album).await?;

    // Create sample tracks
    for i in 1..=5 {
        let track = Track {
            id: Uuid::new_v4(),
            title: format!("Track {}", i),
            album_id: Some(album.id),
            artist_id: Some(artist.id),
            duration: 180 + (i * 10),
            file_path: format!("/music/track{}.mp3", i),
            file_size: 5000000 + (i as u64 * 100000),
            bitrate: 320,
            format: "mp3".to_string(),
            track_number: Some(i),
            disc_number: Some(1),
            year: Some(2024),
            genre: Some("Demo".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        storage.save_track(&track).await?;
    }

    tracing::info!("Sample data added: 1 artist, 1 album, 5 tracks");

    Ok(())
}
