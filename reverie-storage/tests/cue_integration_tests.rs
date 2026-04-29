//! Integration tests for CUE scanner functionality
//!
//! Verifies that CUE sheet scanning, embedded CUESHEET extraction, rescan
//! handling, and error cases properly populate the database.
//!
//! Each test:
//! - Creates a temp directory with audio files + optional .cue files
//! - Uses DatabaseConfig (in-memory SQLite) with VFS pointing to temp dir
//! - Calls `perform_scan()` to scan and populate the database
//! - Queries the tracks table directly via sqlx to verify state

use reverie_storage::{DatabaseConfig, DatabaseStorage, Storage, VfsConfig};
use sqlx::Row;

// ============================================================================
// WAV Generation
// ============================================================================

/// Generate a minimal WAV file using hound with a sine wave.
fn make_test_wav(sample_rate: u32, channels: u16, duration_secs: f32) -> Vec<u8> {
    let mut buf = Vec::new();
    let spec = hound::WavSpec {
        channels,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::new(std::io::Cursor::new(&mut buf), spec).unwrap();
    let num_samples = (sample_rate as f32 * duration_secs) as u32;
    for i in 0..num_samples {
        let sample = ((i as f64 * 440.0 * 2.0 * std::f64::consts::PI / sample_rate as f64).sin()
            * 16000.0) as i16;
        for _ in 0..channels {
            writer.write_sample(sample).unwrap();
        }
    }
    writer.finalize().unwrap();
    buf
}

// ============================================================================
// FLAC Binary Generation (for embedded CUESHEET test)
// ============================================================================

fn flac_be24(buf: &mut Vec<u8>, val: u32) {
    buf.push(((val >> 16) & 0xFF) as u8);
    buf.push(((val >> 8) & 0xFF) as u8);
    buf.push((val & 0xFF) as u8);
}

fn flac_be16(buf: &mut Vec<u8>, val: u16) {
    buf.push(((val >> 8) & 0xFF) as u8);
    buf.push((val & 0xFF) as u8);
}

fn flac_be64(buf: &mut Vec<u8>, val: u64) {
    buf.extend_from_slice(&val.to_be_bytes());
}

/// Build a minimal well-formed FLAC file with STREAMINFO, CUESHEET, and VorbisComment.
///
/// `tracks`: (track_number, sample_offset, indices: [(index_number, relative_offset)])
fn make_test_flac(sample_rate: u32, tracks: &[(u8, u64, Vec<(u8, u64)>)]) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(b"fLaC");

    // --- STREAMINFO block (34 bytes) ---
    let ch: u8 = 2;
    let bps: u8 = 16;
    let total_samples: u64 = sample_rate as u64; // ~1 second

    let mut si = Vec::with_capacity(34);
    flac_be16(&mut si, 4096); // min_block_size
    flac_be16(&mut si, 4096); // max_block_size
    si.extend_from_slice(&[0u8; 3]); // min_frame_size
    si.extend_from_slice(&[0u8; 3]); // max_frame_size
    // sample_rate[19:12]
    si.push(((sample_rate >> 12) & 0xFF) as u8);
    // sample_rate[11:4]
    si.push(((sample_rate >> 4) & 0xFF) as u8);
    // sample_rate[3:0] | (ch-1)[2:0] | (bps-1)[4]
    si.push((((sample_rate & 0xF) as u8) << 4) | (((ch - 1) & 0x7) << 1) | (((bps - 1) >> 4) & 0x01));
    // (bps-1)[3:0] | total_samples[35:32]
    si.push((((bps - 1) & 0x0F) << 4) as u8);
    flac_be16(&mut si, ((total_samples >> 16) & 0xFFFF) as u16);
    flac_be16(&mut si, (total_samples & 0xFFFF) as u16);
    si.resize(34, 0); // MD5 zeros

    // STREAMINFO header: type=0, not-last
    buf.push(0x00);
    flac_be24(&mut buf, 34);
    buf.extend_from_slice(&si);

    // --- CUESHEET block (type 5) ---
    let mut cd = Vec::new();
    cd.resize(128, 0); // media_catalog_number (128 bytes NUL-padded)
    flac_be64(&mut cd, 0); // lead_in
    cd.push(0); // flags: is_cd=false
    cd.resize(cd.len() + 258, 0); // reserved 258 bytes
    cd.push(tracks.len() as u8);

    for &(tn, off, ref idxs) in tracks {
        flac_be64(&mut cd, off);
        cd.push(tn);
        cd.resize(cd.len() + 12, 0); // ISRC: 12 bytes NUL
        cd.push(0); // flags: audio, no pre_emphasis
        cd.resize(cd.len() + 13, 0); // reserved 13 bytes
        cd.push(idxs.len() as u8);
        for &(in_, io) in idxs {
            flac_be64(&mut cd, io);
            cd.push(in_);
            cd.resize(cd.len() + 3, 0); // reserved 3 bytes
        }
    }

    // CUESHEET header: type=5, not-last
    buf.push(0x05);
    flac_be24(&mut buf, cd.len() as u32);
    buf.extend_from_slice(&cd);

    // --- VorbisComment block (type 4, last) ---
    let vendor = b"Reverie Test";
    let mut vc = Vec::new();
    vc.extend_from_slice(&(vendor.len() as u32).to_le_bytes());
    vc.extend_from_slice(vendor);
    vc.extend_from_slice(&0u32.to_le_bytes()); // 0 user comments

    buf.push(0x84); // type=4, last=true
    flac_be24(&mut buf, vc.len() as u32);
    buf.extend_from_slice(&vc);

    buf
}

// ============================================================================
// Test Harness
// ============================================================================

/// Creates a temp directory with the given files, then constructs a
/// `DatabaseStorage` with in-memory SQLite and a local-filesystem VFS
/// pointing to that temp directory. Returns (TempDir, DatabaseStorage).
async fn setup(files: &[(&str, &[u8])]) -> (tempfile::TempDir, DatabaseStorage) {
    let dir = tempfile::tempdir().unwrap();
    for (name, data) in files {
        let path = dir.path().join(name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::write(&path, data).unwrap();
    }
    let root = dir.path().to_string_lossy().to_string();
    let config = DatabaseConfig::new(":memory:", VfsConfig::local(root));
    let storage = DatabaseStorage::new(config).await.unwrap();
    storage.initialize().await.unwrap();
    (dir, storage)
}

/// Count all tracks in the database.
async fn count_tracks(storage: &DatabaseStorage) -> i64 {
    sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM tracks")
        .fetch_one(storage.pool())
        .await
        .unwrap()
}

/// Count virtual CUE tracks (is_cue_virtual = 1).
async fn count_virtual_tracks(storage: &DatabaseStorage) -> i64 {
    sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM tracks WHERE is_cue_virtual = 1")
        .fetch_one(storage.pool())
        .await
        .unwrap()
}

/// Count non-virtual tracks.
async fn count_physical_tracks(storage: &DatabaseStorage) -> i64 {
    sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM tracks WHERE is_cue_virtual = 0")
        .fetch_one(storage.pool())
        .await
        .unwrap()
}

/// Fetch all virtual track titles, sorted by track_number.
async fn virtual_track_titles(storage: &DatabaseStorage) -> Vec<String> {
    let rows = sqlx::query(
        "SELECT title FROM tracks WHERE is_cue_virtual = 1 ORDER BY track_number",
    )
    .fetch_all(storage.pool())
    .await
    .unwrap();
    rows.iter().map(|r| r.get::<String, _>("title")).collect()
}

/// Fetch source_file values for virtual tracks.
async fn virtual_track_source_files(storage: &DatabaseStorage) -> Vec<String> {
    let rows = sqlx::query(
        "SELECT source_file FROM tracks WHERE is_cue_virtual = 1 ORDER BY track_number",
    )
    .fetch_all(storage.pool())
    .await
    .unwrap();
    rows.iter()
        .map(|r| r.get::<String, _>("source_file"))
        .collect()
}

/// Fetch cue_path values for all tracks.
async fn cue_paths(storage: &DatabaseStorage) -> Vec<Option<String>> {
    let rows = sqlx::query("SELECT cue_path FROM tracks ORDER BY track_number")
        .fetch_all(storage.pool())
        .await
        .unwrap();
    rows.iter()
        .map(|r| r.get::<Option<String>, _>("cue_path"))
        .collect()
}

// ============================================================================
// Tests
// ============================================================================

/// Test 1: Scan directory with FLAC + .cue produces N+1 tracks.
///
/// Creates a WAV file with a sidecar .cue containing 4 tracks.
/// After scanning, expects 5 tracks total: 1 whole-disc + 4 virtual tracks.
#[tokio::test]
async fn test_scan_cue_album_produces_correct_track_count() {
    // 60-second WAV at 44100 Hz, stereo, 16-bit
    let wav = make_test_wav(44100, 2, 60.0);

    let cue = r#"PERFORMER "Test Artist"
TITLE "Test Album"
FILE "album.wav" WAVE
  TRACK 01 AUDIO
    TITLE "Opening"
    PERFORMER "Test Artist"
    INDEX 01 00:00:00
  TRACK 02 AUDIO
    TITLE "Development"
    PERFORMER "Test Artist"
    INDEX 01 00:15:00
  TRACK 03 AUDIO
    TITLE "Climax"
    PERFORMER "Test Artist"
    INDEX 01 00:30:00
  TRACK 04 AUDIO
    TITLE "Resolution"
    PERFORMER "Test Artist"
    INDEX 01 00:45:00
"#;

    let (_dir, storage) = setup(&[("album.wav", &wav), ("album.cue", cue.as_bytes())]).await;

    storage.perform_scan("").await.expect("scan should succeed");

    assert_eq!(count_tracks(&storage).await, 5, "expected 5 tracks (1 whole + 4 virtual)");
    assert_eq!(count_virtual_tracks(&storage).await, 4, "expected 4 virtual tracks");
    assert_eq!(count_physical_tracks(&storage).await, 1, "expected 1 whole-disc track");

    let titles = virtual_track_titles(&storage).await;
    assert_eq!(titles, vec!["Opening", "Development", "Climax", "Resolution"]);

    // All virtual tracks should point to album.cue
    let paths = cue_paths(&storage).await;
    let virt_paths: Vec<_> = paths.iter().filter(|p| p.is_some()).collect();
    assert_eq!(virt_paths.len(), 4);
    for p in virt_paths {
        assert!(p.as_deref().unwrap().ends_with("album.cue"));
    }
}

/// Test 2: FLAC with embedded CUESHEET produces same result as external .cue.
///
/// Creates a FLAC file with 3 embedded CUESHEET tracks (no sidecar .cue).
/// After scanning, expects 4 tracks total: 1 whole-disc + 3 virtual tracks.
#[tokio::test]
async fn test_scan_embedded_cuesheet_same_as_external() {
    let sample_rate: u32 = 48000;

    // FLAC with 3 embedded tracks at 0s, 1s, and 2s
    let flac_tracks = vec![
        (1u8, 0u64, vec![(1u8, 0u64)]),
        (2, sample_rate as u64, vec![(1, 0)]),
        (3, sample_rate as u64 * 2, vec![(1, 0)]),
    ];
    let flac = make_test_flac(sample_rate, &flac_tracks);

    let (_dir, storage) = setup(&[("album.flac", &flac)]).await;

    storage.perform_scan("").await.expect("scan should succeed");

    assert_eq!(count_tracks(&storage).await, 4, "expected 4 tracks (1 whole + 3 virtual)");
    assert_eq!(count_virtual_tracks(&storage).await, 3, "expected 3 virtual tracks");
    assert_eq!(count_physical_tracks(&storage).await, 1, "expected 1 whole-disc track");

    // Virtual tracks should reference the FLAC as source_file
    let sources = virtual_track_source_files(&storage).await;
    assert_eq!(sources.len(), 3);
    for s in &sources {
        assert!(s.ends_with("album.flac"), "source_file should be album.flac, got {}", s);
    }
}

/// Test 3: Scan regular audio file (no .cue) produces exactly 1 track.
#[tokio::test]
async fn test_scan_no_cue_regression() {
    let wav = make_test_wav(44100, 2, 10.0);

    let (_dir, storage) = setup(&[("track.wav", &wav)]).await;

    storage.perform_scan("").await.expect("scan should succeed");

    assert_eq!(count_tracks(&storage).await, 1, "expected exactly 1 track for file without CUE");
    assert_eq!(count_virtual_tracks(&storage).await, 0, "expected 0 virtual tracks");
    assert_eq!(count_physical_tracks(&storage).await, 1, "expected 1 whole-disc track");

    // Verify cue_path is NULL
    let row = sqlx::query("SELECT cue_path, is_cue_virtual FROM tracks LIMIT 1")
        .fetch_one(storage.pool())
        .await
        .unwrap();
    let cue_path: Option<String> = row.get("cue_path");
    let is_virtual: i64 = row.get("is_cue_virtual");
    assert!(cue_path.is_none(), "cue_path should be NULL for non-CUE track");
    assert_eq!(is_virtual, 0, "is_cue_virtual should be 0");
}

/// Test 4: Modify .cue and rescan — track count updates, no duplicates.
///
/// First scans with a 2-track CUE → expects 3 tracks.
/// Then rewrites the CUE with 3 tracks and rescans → expects 4 tracks total.
/// Stable UUIDs ensure no duplicate tracks.
#[tokio::test]
async fn test_rescan_modified_cue_no_duplicates() {
    let wav = make_test_wav(44100, 2, 60.0);

    // CUE with 2 tracks
    let cue_v1 = r#"PERFORMER "Artist"
TITLE "Album"
FILE "album.wav" WAVE
  TRACK 01 AUDIO
    TITLE "Track A"
    INDEX 01 00:00:00
  TRACK 02 AUDIO
    TITLE "Track B"
    INDEX 01 00:15:00
"#;

    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("album.wav"), &wav).unwrap();
    std::fs::write(dir.path().join("album.cue"), cue_v1.as_bytes()).unwrap();

    let root = dir.path().to_string_lossy().to_string();
    let config = DatabaseConfig::new(":memory:", VfsConfig::local(root));
    let storage = DatabaseStorage::new(config).await.unwrap();
    storage.initialize().await.unwrap();

    // First scan
    storage.perform_scan("").await.expect("first scan");
    assert_eq!(count_tracks(&storage).await, 3, "after first scan: 3 tracks (1 whole + 2 virtual)");
    assert_eq!(count_virtual_tracks(&storage).await, 2);

    // Modify CUE to have 3 tracks
    let cue_v2 = r#"PERFORMER "Artist"
TITLE "Album"
FILE "album.wav" WAVE
  TRACK 01 AUDIO
    TITLE "Track A"
    INDEX 01 00:00:00
  TRACK 02 AUDIO
    TITLE "Track B"
    INDEX 01 00:15:00
  TRACK 03 AUDIO
    TITLE "Track C"
    INDEX 01 00:30:00
"#;
    std::fs::write(dir.path().join("album.cue"), cue_v2.as_bytes()).unwrap();

    // Second scan (rescan)
    storage.perform_scan("").await.expect("second scan");
    assert_eq!(count_tracks(&storage).await, 4, "after rescan: 4 tracks (1 whole + 3 virtual)");
    assert_eq!(count_virtual_tracks(&storage).await, 3);

    // Verify no duplicates — unique IDs in tracks table
    let id_count: i64 = sqlx::query_scalar("SELECT COUNT(DISTINCT id) FROM tracks")
        .fetch_one(storage.pool())
        .await
        .unwrap();
    assert_eq!(id_count, count_tracks(&storage).await, "all track IDs should be unique (no duplicates)");

    let titles = virtual_track_titles(&storage).await;
    assert_eq!(titles, vec!["Track A", "Track B", "Track C"]);
}

/// Test 5: Multiple audio files with separate .cue files in the same dir.
///
/// Each audio file gets its own virtual tracks, and the totals add up correctly.
#[tokio::test]
async fn test_scan_multiple_cues_in_same_dir() {
    let wav_a = make_test_wav(44100, 2, 30.0);
    let wav_b = make_test_wav(44100, 2, 45.0);

    let cue_a = r#"PERFORMER "Artist A"
TITLE "Album A"
FILE "a.wav" WAVE
  TRACK 01 AUDIO
    TITLE "A1"
    INDEX 01 00:00:00
  TRACK 02 AUDIO
    TITLE "A2"
    INDEX 01 00:10:00
"#;

    let cue_b = r#"PERFORMER "Artist B"
TITLE "Album B"
FILE "b.wav" WAVE
  TRACK 01 AUDIO
    TITLE "B1"
    INDEX 01 00:00:00
  TRACK 02 AUDIO
    TITLE "B2"
    INDEX 01 00:15:00
  TRACK 03 AUDIO
    TITLE "B3"
    INDEX 01 00:30:00
"#;

    let (_dir, storage) = setup(&[
        ("a.wav", &wav_a),
        ("a.cue", cue_a.as_bytes()),
        ("b.wav", &wav_b),
        ("b.cue", cue_b.as_bytes()),
    ])
    .await;

    storage.perform_scan("").await.expect("scan should succeed");

    // Expected: (1 whole + 2 virt from a) + (1 whole + 3 virt from b) = 7
    assert_eq!(count_tracks(&storage).await, 7, "expected 7 tracks total");
    assert_eq!(count_virtual_tracks(&storage).await, 5, "expected 5 virtual tracks");
    assert_eq!(count_physical_tracks(&storage).await, 2, "expected 2 whole-disc tracks");

    // Check per-file virtual track counts
    let a_virt: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM tracks WHERE is_cue_virtual = 1 AND source_file LIKE '%a.wav'",
    )
    .fetch_one(storage.pool())
    .await
    .unwrap();
    assert_eq!(a_virt, 2, "a.wav should have 2 virtual tracks");

    let b_virt: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM tracks WHERE is_cue_virtual = 1 AND source_file LIKE '%b.wav'",
    )
    .fetch_one(storage.pool())
    .await
    .unwrap();
    assert_eq!(b_virt, 3, "b.wav should have 3 virtual tracks");
}

/// Test 6: .cue FILE directive references non-existent file — handled gracefully.
///
/// The CUE sidecar references an audio file that doesn't exist on disk, but
/// the scanner associates the .cue by stem with the actual audio file being
/// scanned, so virtual tracks are still created successfully.
#[tokio::test]
async fn test_scan_cue_file_not_found() {
    let wav = make_test_wav(44100, 2, 20.0);

    // CUE FILE directive points to "missing.flac" (doesn't exist)
    let cue = r#"PERFORMER "Artist"
TITLE "Album"
FILE "missing.flac" WAVE
  TRACK 01 AUDIO
    TITLE "Ghost Track 1"
    INDEX 01 00:00:00
  TRACK 02 AUDIO
    TITLE "Ghost Track 2"
    INDEX 01 00:10:00
"#;

    let (_dir, storage) = setup(&[("track.wav", &wav), ("track.cue", cue.as_bytes())]).await;

    // Scan should succeed without error
    let result = storage.perform_scan("").await;
    assert!(result.is_ok(), "scan should succeed even when CUE FILE references missing file");

    // Virtual tracks still created from the CUE data
    assert_eq!(count_tracks(&storage).await, 3, "expected 3 tracks (1 whole + 2 virtual)");
    assert_eq!(count_virtual_tracks(&storage).await, 2);

    let titles = virtual_track_titles(&storage).await;
    assert_eq!(titles, vec!["Ghost Track 1", "Ghost Track 2"]);

    // Virtual tracks are associated with the actual scanned file (track.wav), not the missing one
    let sources = virtual_track_source_files(&storage).await;
    for s in &sources {
        assert!(s.ends_with("track.wav"), "virtual track should reference real audio file, got {}", s);
    }
}
