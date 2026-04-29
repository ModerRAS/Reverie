//! CUE sheet parser
//!
//! Parses external `.cue` files into structured Rust types using the
//! [`rcue`](https://crates.io/crates/rcue) crate (MIT, zero-dependency, pure Rust).
//!
//! Handles UTF-8 BOM stripping, blank-line filtering (required for strict-mode
//! parsing), and converts INDEX times from MM:SS:FF format to seconds.

use std::io::{BufReader, Cursor};
use std::path::Path;

use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::error::{Result, StorageError};

/// Namespace UUID for stable track ID generation.
/// Using DNS namespace as base for reverie-specific stable IDs.
const NAMESPACE_REVERIE: Uuid = Uuid::NAMESPACE_DNS;

/// Represents a single track in a CUE sheet.
#[derive(Debug, Clone)]
pub struct CueTrack {
    /// Track number as a string (e.g. "01", "02").
    pub track_number: String,
    /// Title of the track, if present.
    pub title: Option<String>,
    /// Performer of the track, if present (falls back to album performer).
    pub performer: Option<String>,
    /// Start time in seconds (from INDEX 01).
    pub index_start: f64,
    /// Pregap time in seconds (from INDEX 00), if present.
    pub index_pregap: Option<f64>,
    /// Audio file name from the CUE sheet's FILE field.
    pub file_name: String,
}

/// Represents a parsed CUE sheet.
#[derive(Debug, Clone)]
pub struct CueSheet {
    /// Album title from the CUE sheet.
    pub album_title: Option<String>,
    /// Album performer from the CUE sheet.
    pub album_performer: Option<String>,
    /// Tracks listed in the CUE sheet.
    pub tracks: Vec<CueTrack>,
    /// Path to the audio file referenced by the CUE sheet.
    pub file_path: String,
}

/// Generate a stable, deterministic UUID for a virtual track based on file path and track index.
///
/// Uses SHA256 of the canonical file path combined with track index to generate a UUID v5.
/// This ensures the same file+track combination always produces the same UUID across scans.
///
/// # Arguments
/// * `file_path` - The path to the audio file
/// * `track_index` - The track index (0 = whole disc, 1+ = individual tracks)
///
/// # Returns
/// A UUID v5 string that is deterministic for the given inputs
pub fn stable_uuid(file_path: &str, track_index: u32) -> String {
    // Canonicalize the path to get absolute path for consistency
    let canonical_path = std::path::Path::new(file_path)
        .canonicalize()
        .unwrap_or_else(|_| std::path::PathBuf::from(file_path));

    let path_str = canonical_path.to_string_lossy();

    // Create input string: "canonical_path:track_index"
    let input = format!("{}:{}", path_str, track_index);

    // Generate SHA256 hash of the input
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let hash_bytes = hasher.finalize();

    // Use UUID v5 with the hash bytes (only first 16 bytes needed for UUID)
    let uuid_v5 = Uuid::new_v5(&NAMESPACE_REVERIE, &hash_bytes[..16]);

    uuid_v5.to_string()
}

/// Parse a CUE file at the given path into a [`CueSheet`].
///
/// Uses strict-mode parsing after filtering blank lines and stripping the
/// UTF-8 BOM, so malformed CUE files produce an `Err` instead of silently
/// skipping bad entries.
///
/// # Errors
///
/// Returns `StorageError::IoError` if the file cannot be read, or
/// `StorageError::Unavailable` if the CUE content cannot be parsed.
pub fn parse_cue_file(path: &Path) -> Result<CueSheet> {
    let raw = std::fs::read_to_string(path).map_err(StorageError::IoError)?;

    // Strip UTF-8 BOM — rcue does not support it natively.
    let content = raw.strip_prefix('\u{FEFF}').unwrap_or(&raw);

    // Filter blank/whitespace-only lines so strict-mode parsing succeeds.
    // rcue's strict mode rejects empty lines as "unknown tokens".
    let filtered: String = content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<&str>>()
        .join("\n");

    let cursor = Cursor::new(filtered.as_bytes());
    let mut buf_reader = BufReader::new(cursor);

    let cue = rcue::parser::parse(&mut buf_reader, true)
        .map_err(|e| StorageError::Unavailable(format!("Failed to parse CUE file: {}", e)))?;

    // Build the CueSheet from rcue's Cue struct
    let album_title = cue.title;
    let album_performer = cue.performer;

    // Use the first FILE path as the sheet-level file_path
    let file_path = cue
        .files
        .first()
        .map(|f| f.file.clone())
        .unwrap_or_default();

    let mut tracks = Vec::new();

    for file in &cue.files {
        let file_name = file.file.clone();
        for track in &file.tracks {
            // Extract INDEX 00 (pregap) and INDEX 01 (start time)
            let mut index_start = 0.0f64;
            let mut index_pregap: Option<f64> = None;

            for (index_num, duration) in &track.indices {
                let seconds = duration.as_secs_f64();
                match index_num.as_str() {
                    "00" => index_pregap = Some(seconds),
                    "01" => index_start = seconds,
                    _ => {} // INDEX 02+ handled elsewhere if needed
                }
            }

            tracks.push(CueTrack {
                track_number: track.no.clone(),
                title: track.title.clone(),
                performer: track.performer.clone(),
                index_start,
                index_pregap,
                file_name: file_name.clone(),
            });
        }
    }

    Ok(CueSheet {
        album_title,
        album_performer,
        tracks,
        file_path,
    })
}

// ---------------------------------------------------------------------------
// FLAC embedded CUESHEET extraction (feature = "flac")
// ---------------------------------------------------------------------------

/// Extract an embedded CUESHEET block (FLAC block type 5) from a FLAC file.
///
/// Uses the [`flac`] crate to read the binary CUESHEET metadata block.  The
/// block contains only sample offsets, index points, ISRC codes and track
/// numbers — **no text titles or performer names**.
///
/// Returns `Ok(None)` if the FLAC file has no CUESHEET block (not an error).
#[cfg(feature = "flac")]
pub fn extract_embedded_cue_from_flac(flac_path: &Path) -> Result<Option<CueSheet>> {
    let path_str = flac_path
        .to_str()
        .ok_or_else(|| StorageError::InvalidPath(format!("Non-UTF8 path: {:?}", flac_path)))?;

    // Read STREAMINFO to get the sample rate for offset→seconds conversion.
    let stream_info = flac::metadata::get_stream_info(path_str).map_err(|e| {
        StorageError::Unavailable(format!(
            "Failed to read FLAC stream info from {}: {:?}",
            path_str, e
        ))
    })?;
    let sample_rate = stream_info.sample_rate as f64;

    // Read the CUESHEET block (type 5).
    let flac_cue = match flac::metadata::get_cue_sheet(path_str) {
        Ok(cue) => cue,
        Err(flac::ErrorKind::NotFound) => return Ok(None),
        Err(e) => {
            return Err(StorageError::Unavailable(format!(
                "Failed to read FLAC CUESHEET from {}: {:?}",
                path_str, e
            )))
        }
    };

    let file_name = flac_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let mut tracks = Vec::with_capacity(flac_cue.tracks.len());

    for t in &flac_cue.tracks {
        // Track number is a u8 — format as zero-padded two-digit string.
        let track_number = format!("{:02}", t.number);

        // INDEX points in FLAC CUESHEET: the *index offset* is relative to the
        // *track offset*, so the absolute sample position is track.offset + idx.offset.
        let mut index_start = 0.0_f64;
        let mut index_pregap: Option<f64> = None;

        for idx in &t.indices {
            let abs_offset = t.offset + idx.offset;
            let seconds = abs_offset as f64 / sample_rate;
            match idx.number {
                0 => index_pregap = Some(seconds),
                1 => index_start = seconds,
                _ => {} // INDEX 02+ can be handled elsewhere if needed
            }
        }

        // If there was no INDEX 01 (shouldn't happen per FLAC spec), fall back
        // to the track offset itself.
        if t.indices.iter().all(|i| i.number != 1) {
            index_start = t.offset as f64 / sample_rate;
        }

        tracks.push(CueTrack {
            track_number,
            title: None,     // CUESHEET block is binary — no text
            performer: None, // CUESHEET block is binary — no text
            index_start,
            index_pregap,
            file_name: file_name.clone(),
        });
    }

    Ok(Some(CueSheet {
        album_title: None,     // CUESHEET block has no album title
        album_performer: None, // CUESHEET block has no album performer
        tracks,
        file_path: path_str.to_string(),
    }))
}

/// Try to extract a [`CueSheet`] for the audio file at `path`.
///
/// Strategy (in order):
/// 1. If the file is FLAC with an embedded CUESHEET block, parse it.
/// 2. Look for a sidecar `.cue` file with the same stem as `path`.
///
/// Returns `Ok(None)` when neither source is available.
#[cfg(feature = "flac")]
pub fn path_to_cue_sheet(path: &Path) -> Result<Option<CueSheet>> {
    // 1. Try FLAC embedded CUESHEET first.
    if let Some(cue) = extract_embedded_cue_from_flac(path)? {
        return Ok(Some(cue));
    }

    // 2. Check for external .cue file sidecar.
    let cue_path = path.with_extension("cue");
    if cue_path.exists() {
        let sheet = parse_cue_file(&cue_path)?;
        return Ok(Some(sheet));
    }

    Ok(None)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::path::Path;

    /// Builds the absolute path to a CUE test fixture.
    fn fixture_path(name: &str) -> String {
        let manifest = env!("CARGO_MANIFEST_DIR");
        format!("{}/../tests/fixtures/audio/cue/{}", manifest, name)
    }

    #[test]
    fn test_parse_basic_cue() {
        let sheet = parse_cue_file(Path::new(&fixture_path("basic.cue"))).unwrap();

        assert_eq!(sheet.album_title.as_deref(), Some("Test Album"));
        assert_eq!(sheet.album_performer.as_deref(), Some("Test Artist"));
        assert_eq!(sheet.file_path, "album.flac");
        assert_eq!(sheet.tracks.len(), 8);

        // First track: "Opening Movement" at 00:00:00
        let first = &sheet.tracks[0];
        assert_eq!(first.track_number, "01");
        assert_eq!(first.title.as_deref(), Some("Opening Movement"));
        assert_eq!(first.performer.as_deref(), Some("Test Artist"));
        assert!((first.index_start - 0.0).abs() < f64::EPSILON);
        assert!(first.index_pregap.is_none());
        assert_eq!(first.file_name, "album.flac");

        // Last track: "Hidden Track" at 23:30:00 = 1410.0 s
        let last = &sheet.tracks[7];
        assert_eq!(last.track_number, "08");
        assert_eq!(last.title.as_deref(), Some("Hidden Track"));
        let expected = 23.0 * 60.0 + 30.0; // 1410.0
        assert!((last.index_start - expected).abs() < f64::EPSILON);
    }

    #[test]
    fn test_parse_pregap_cue() {
        let sheet = parse_cue_file(Path::new(&fixture_path("with_pregap.cue"))).unwrap();

        assert_eq!(sheet.album_title.as_deref(), Some("Album With Pregaps"));
        assert_eq!(sheet.tracks.len(), 3);

        // Track 1: INDEX 00 @ 00:00:00, INDEX 01 @ 00:02:00
        let t1 = &sheet.tracks[0];
        assert_eq!(t1.track_number, "01");
        assert_eq!(t1.title.as_deref(), Some("Track One"));
        assert!((t1.index_start - 2.0).abs() < f64::EPSILON);
        assert!(t1.index_pregap.is_some());
        assert!((t1.index_pregap.unwrap() - 0.0).abs() < f64::EPSILON);

        // Track 2: INDEX 00 @ 04:00:00, INDEX 01 @ 04:02:00
        let t2 = &sheet.tracks[1];
        assert_eq!(t2.track_number, "02");
        assert_eq!(t2.title.as_deref(), Some("Track Two"));
        assert!((t2.index_start - 242.0).abs() < f64::EPSILON); // 4*60+2
        assert!(t2.index_pregap.is_some());
        assert!((t2.index_pregap.unwrap() - 240.0).abs() < f64::EPSILON); // 4*60

        // Track 3: only INDEX 01 @ 08:00:00
        let t3 = &sheet.tracks[2];
        assert_eq!(t3.track_number, "03");
        assert_eq!(t3.title.as_deref(), Some("Track Three"));
        assert!((t3.index_start - 480.0).abs() < f64::EPSILON); // 8*60
        assert!(t3.index_pregap.is_none());
    }

    #[test]
    fn test_parse_malformed_cue() {
        let result = parse_cue_file(Path::new(&fixture_path("malformed.cue")));
        assert!(result.is_err(), "Malformed CUE must produce an error");

        let err = result.unwrap_err();
        let msg = format!("{}", err);
        // Verify it's not a panic — it's a proper StorageError
        assert!(
            msg.contains("Failed to parse CUE file") || msg.contains("strict mode"),
            "Error message should indicate parse failure: {}",
            msg
        );
    }

    #[test]
    fn test_index_time_conversion() {
        // rcue converts MM:SS:FF using frames/75.
        // Verify by parsing a known CUE and checking time values.
        let sheet = parse_cue_file(Path::new(&fixture_path("basic.cue"))).unwrap();

        // Track 2: INDEX 01 03:45:00 → 3*60 + 45 = 225.0 s
        let t2 = &sheet.tracks[1];
        let expected: f64 = 3.0 * 60.0 + 45.0;
        assert!(
            (t2.index_start - expected).abs() < f64::EPSILON,
            "Expected {} s, got {} s",
            expected,
            t2.index_start
        );

        // Track 4: INDEX 01 11:15:00 → 11*60 + 15 = 675.0 s
        let t4 = &sheet.tracks[3];
        let expected: f64 = 11.0 * 60.0 + 15.0;
        assert!(
            (t4.index_start - expected).abs() < f64::EPSILON,
            "Expected {} s, got {} s",
            expected,
            t4.index_start
        );

        // Track 5: INDEX 01 14:00:00 → 14*60 = 840.0 s
        let t5 = &sheet.tracks[4];
        let expected: f64 = 14.0 * 60.0;
        assert!(
            (t5.index_start - expected).abs() < f64::EPSILON,
            "Expected {} s, got {} s",
            expected,
            t5.index_start
        );
    }

    #[test]
    fn test_parse_missing_metadata() {
        let sheet = parse_cue_file(Path::new(&fixture_path("missing_metadata.cue"))).unwrap();

        assert_eq!(sheet.album_title.as_deref(), Some("Partial Metadata Album"));
        assert_eq!(
            sheet.album_performer.as_deref(),
            Some("Full Metadata Artist")
        );
        assert_eq!(sheet.tracks.len(), 4);

        // Track 1: complete metadata
        let t1 = &sheet.tracks[0];
        assert_eq!(t1.title.as_deref(), Some("Complete Track"));
        assert_eq!(t1.performer.as_deref(), Some("Full Metadata Artist"));

        // Track 2: missing TITLE and PERFORMER
        let t2 = &sheet.tracks[1];
        assert_eq!(t2.track_number, "02");
        assert!(t2.title.is_none());
        assert!(t2.performer.is_none());
        assert!((t2.index_start - 180.0).abs() < f64::EPSILON); // 3:00

        // Track 3: missing PERFORMER
        let t3 = &sheet.tracks[2];
        assert_eq!(t3.title.as_deref(), Some("Only Title"));
        assert!(t3.performer.is_none());

        // Track 4: missing TITLE and PERFORMER
        let t4 = &sheet.tracks[3];
        assert_eq!(t4.track_number, "04");
        assert!(t4.title.is_none());
        assert!(t4.performer.is_none());
    }

    #[test]
    fn test_empty_path() {
        let result = parse_cue_file(Path::new("nonexistent_file_12345.cue"));
        assert!(result.is_err(), "Non-existent file must produce an error");

        let err = result.unwrap_err();
        match err {
            StorageError::IoError(_) => {} // expected
            other => panic!("Expected IoError, got: {}", other),
        }
    }

    // ---------------------------------------------------------------------------
    // stable_uuid tests
    // ---------------------------------------------------------------------------

    #[test]
    fn test_stable_uuid_deterministic() {
        // Two calls with same input should produce same UUID
        let path = "test_audio.flac";
        let index = 1;

        let uuid1 = stable_uuid(path, index);
        let uuid2 = stable_uuid(path, index);

        assert_eq!(uuid1, uuid2, "stable_uuid should be deterministic");
        assert!(!uuid1.is_empty(), "UUID should not be empty");
    }

    #[test]
    fn test_stable_uuid_different_tracks() {
        // Different track indices should produce different UUIDs
        let path = "test_audio.flac";

        let uuid_track0 = stable_uuid(path, 0);
        let uuid_track1 = stable_uuid(path, 1);
        let uuid_track2 = stable_uuid(path, 2);

        assert_ne!(
            uuid_track0, uuid_track1,
            "Different tracks should have different UUIDs"
        );
        assert_ne!(
            uuid_track1, uuid_track2,
            "Different tracks should have different UUIDs"
        );
        assert_ne!(
            uuid_track0, uuid_track2,
            "Different tracks should have different UUIDs"
        );
    }

    #[test]
    fn test_stable_uuid_different_files() {
        // Different file paths should produce different UUIDs
        let file1 = "album1/track.flac";
        let file2 = "album2/track.flac";

        let uuid1 = stable_uuid(file1, 1);
        let uuid2 = stable_uuid(file2, 1);

        assert_ne!(
            uuid1, uuid2,
            "Different files should have different UUIDs for same track index"
        );
    }

    #[test]
    fn test_stable_uuid_matches_across_restarts() {
        // Simulate: scan -> delete -> scan -> same UUIDs
        let path = "music/album.flac";

        // First scan
        let uuid_whole_disc1 = stable_uuid(path, 0);
        let uuid_track1_first = stable_uuid(path, 1);

        // Simulate restart/delete
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Second scan
        let uuid_whole_disc2 = stable_uuid(path, 0);
        let uuid_track1_second = stable_uuid(path, 1);

        // Should be identical across "restarts"
        assert_eq!(
            uuid_whole_disc1, uuid_whole_disc2,
            "Whole-disc UUID should be stable across restarts"
        );
        assert_eq!(
            uuid_track1_first, uuid_track1_second,
            "Track UUID should be stable across restarts"
        );
    }

    #[test]
    fn test_stable_uuid_format() {
        // UUID v5 format: xxxxxxxx-xxxx-5xxx-yxxx-xxxxxxxxxxxx (5 in version position)
        let uuid = stable_uuid("test.flac", 1);

        assert_eq!(uuid.len(), 36, "UUID should be 36 characters");
        assert!(uuid.contains('-'), "UUID should contain hyphens");

        // Check version (5xxx)
        let version_char = uuid.chars().nth(14).unwrap();
        assert_eq!(
            version_char, '5',
            "UUID v5 should have '5' at position 14 (version)"
        );
    }

    #[test]
    fn test_stable_uuid_whole_disc_uses_zero() {
        // Whole-disc track should use track_index 0
        let path = "test.flac";

        let uuid = stable_uuid(path, 0);
        let uuid_track1 = stable_uuid(path, 1);

        assert_ne!(
            uuid, uuid_track1,
            "Whole-disc (0) and track 1 should have different UUIDs"
        );
    }

    // -----------------------------------------------------------------------
    // FLAC CUESHEET tests (feature = "flac")
    // -----------------------------------------------------------------------

    #[cfg(feature = "flac")]
    pub(crate) fn push_be24(buf: &mut Vec<u8>, val: u32) {
        buf.push(((val >> 16) & 0xFF) as u8);
        buf.push(((val >> 8) & 0xFF) as u8);
        buf.push((val & 0xFF) as u8);
    }

    #[cfg(feature = "flac")]
    pub(crate) fn push_be16(buf: &mut Vec<u8>, val: u16) {
        buf.push(((val >> 8) & 0xFF) as u8);
        buf.push((val & 0xFF) as u8);
    }

    #[cfg(feature = "flac")]
    pub(crate) fn push_be64(buf: &mut Vec<u8>, val: u64) {
        buf.extend_from_slice(&val.to_be_bytes());
    }

    /// Write a FLAC CUESHEET block in the exact binary format expected by the
    /// parser (media_catalog_number padded to 128 bytes, 258 reserved bytes
    /// after flags, 13 reserved bytes per track, 3 reserved bytes per index).
    #[cfg(feature = "flac")]
    pub(crate) fn write_flac_cue_block(buf: &mut Vec<u8>, cue: &flac::metadata::CueSheet) {
        // Media catalog number: 128 bytes, ASCII NUL-padded
        let mcn_bytes = cue.media_catalog_number.as_bytes();
        let mcn_len = mcn_bytes.len().min(128);
        buf.extend_from_slice(&mcn_bytes[..mcn_len]);
        if mcn_len < 128 {
            buf.resize(buf.len() + (128 - mcn_len), 0);
        }

        // Lead-in samples: 8 bytes BE
        push_be64(buf, cue.lead_in);

        // Flags: 1 byte (bit 7 = is_cd)
        let mut flags: u8 = 0;
        if cue.is_cd {
            flags |= 0x80;
        }
        buf.push(flags);

        // Reserved: 258 bytes
        buf.resize(buf.len() + 258, 0);

        // Number of tracks: 1 byte
        buf.push(cue.tracks.len() as u8);

        // Tracks
        for track in &cue.tracks {
            push_be64(buf, track.offset);
            buf.push(track.number);

            // ISRC: 12 bytes, NUL-padded
            let isrc_bytes = track.isrc.as_bytes();
            let isrc_len = isrc_bytes.len().min(12);
            buf.extend_from_slice(&isrc_bytes[..isrc_len]);
            if isrc_len < 12 {
                buf.resize(buf.len() + (12 - isrc_len), 0);
            }

            // Flags: bit 7 = is_audio (0 = audio), bit 6 = pre_emphasis
            let mut tflags: u8 = 0;
            if !track.is_audio {
                tflags |= 0x80;
            }
            if track.is_pre_emphasis {
                tflags |= 0x40;
            }
            buf.push(tflags);

            // Reserved: 13 bytes
            buf.resize(buf.len() + 13, 0);

            // Number of indices
            buf.push(track.indices.len() as u8);

            // Indices
            for idx in &track.indices {
                push_be64(buf, idx.offset);
                buf.push(idx.number);
                // Reserved: 3 bytes
                buf.resize(buf.len() + 3, 0);
            }
        }
    }

    /// Build a minimal valid FLAC file with STREAMINFO, optional CUESHEET,
    /// and a VorbisComment block (so the file is well-formed).
    #[cfg(feature = "flac")]
    pub(crate) fn make_test_flac(
        sample_rate: u32,
        flac_cue: Option<&flac::metadata::CueSheet>,
    ) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(b"fLaC");

        // --- STREAMINFO block (34 bytes) ---
        // Uses the exact same byte layout as generate_flac() in
        // src/bin/generate_audio_fixtures.rs.
        let mut streaminfo = Vec::with_capacity(34);
        push_be16(&mut streaminfo, 4096); // min block size
        push_be16(&mut streaminfo, 4096); // max block size
        streaminfo.extend_from_slice(&[0u8; 3]); // min frame size
        streaminfo.extend_from_slice(&[0u8; 3]); // max frame size

        // --- sample_rate (20 bits) + channels (3 bits) + bps (5 bits) ---
        // FLAC spec bit layout:
        // byte 10: sr[19:12]
        // byte 11: sr[11:4]
        // byte 12: sr[3:0] | ch[2:0] | bps[4]
        // byte 13: bps[3:0] | ts[35:32]
        let ch: u8 = (2u8 - 1) & 0x7; // channels-1
        let bps: u8 = (16u8 - 1) & 0x1F; // bps-1
        streaminfo.push(((sample_rate >> 12) & 0xFF) as u8);
        streaminfo.push(((sample_rate >> 4) & 0xFF) as u8);
        streaminfo.push((((sample_rate & 0xF) as u8) << 4) | (ch << 1) | ((bps >> 4) & 0x01));
        streaminfo.push(((bps & 0x0F) << 4) as u8);

        // --- total_samples (36 bits) ---
        let ts: u64 = sample_rate as u64; // 1 second of audio
        streaminfo.push(((ts >> 28) & 0xFF) as u8);
        streaminfo.push(((ts >> 20) & 0xFF) as u8);
        streaminfo.push(((ts >> 12) & 0xFF) as u8);
        streaminfo.push(((ts >> 4) & 0xFF) as u8);
        streaminfo.push(((ts & 0xF) as u8) << 4);
        // MD5: 16 bytes of zeros
        streaminfo.resize(34, 0);

        let _has_cue = flac_cue.is_some();

        // STREAMINFO header: type=0, last-block=false (CUESHEET or VorbisComment follows)
        buf.push(0x00);
        push_be24(&mut buf, 34);
        buf.extend_from_slice(&streaminfo);

        // --- Optional CUESHEET block (type 5) ---
        if let Some(cue) = flac_cue {
            let mut cue_data = Vec::new();
            write_flac_cue_block(&mut cue_data, cue);

            // Type=5, last-block=false (VorbisComment follows)
            buf.push(0x05);
            push_be24(&mut buf, cue_data.len() as u32);
            buf.extend_from_slice(&cue_data);
        }

        // --- VorbisComment block (always present for well-formedness) ---
        let vendor = b"Reverie Test";
        let mut vc_data = Vec::new();
        // vendor length (little-endian u32)
        vc_data.extend_from_slice(&(vendor.len() as u32).to_le_bytes());
        vc_data.extend_from_slice(vendor);
        // user comment count = 0
        vc_data.extend_from_slice(&0u32.to_le_bytes());

        // VorbisComment header: type=4, last-block=true
        buf.push(0x84);
        push_be24(&mut buf, vc_data.len() as u32);
        buf.extend_from_slice(&vc_data);

        buf
    }

    #[cfg(feature = "flac")]
    #[test]
    fn test_extract_embedded_cue_from_flac() {
        let sample_rate: u32 = 48000;

        // Build a CUESHEET with 2 tracks using the flac crate's types.
        let flac_cue = flac::metadata::CueSheet {
            media_catalog_number: String::new(),
            lead_in: 0,
            is_cd: false,
            tracks: vec![
                // Track 1 at sample 0, INDEX 01 at relative offset 0
                flac::metadata::CueSheetTrack {
                    offset: 0,
                    number: 1,
                    isrc: String::new(),
                    is_audio: true,
                    is_pre_emphasis: false,
                    indices: vec![flac::metadata::CueSheetTrackIndex {
                        offset: 0,
                        number: 1,
                    }],
                },
                // Track 2 at sample 48000 (1.0 s), INDEX 01 at relative offset 0
                flac::metadata::CueSheetTrack {
                    offset: sample_rate as u64,
                    number: 2,
                    isrc: String::new(),
                    is_audio: true,
                    is_pre_emphasis: false,
                    indices: vec![flac::metadata::CueSheetTrackIndex {
                        offset: 0,
                        number: 1,
                    }],
                },
            ],
        };

        let flac_data = make_test_flac(sample_rate, Some(&flac_cue));
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("test.flac");
        std::fs::write(&path, &flac_data).expect("write FLAC");

        let sheet = extract_embedded_cue_from_flac(&path)
            .expect("extract should succeed")
            .expect("should have CUESHEET");

        assert_eq!(sheet.tracks.len(), 2);
        assert!(sheet.album_title.is_none());
        assert!(sheet.album_performer.is_none());
        assert!(sheet.file_path.ends_with("test.flac"));

        // Track 1
        let t1 = &sheet.tracks[0];
        assert_eq!(t1.track_number, "01");
        assert!(t1.title.is_none());
        assert!(t1.performer.is_none());
        assert!((t1.index_start - 0.0).abs() < f64::EPSILON);
        assert!(t1.index_pregap.is_none());

        // Track 2 — starts at 48000 samples / 48000 Hz = 1.0 s
        let t2 = &sheet.tracks[1];
        assert_eq!(t2.track_number, "02");
        assert!((t2.index_start - 1.0).abs() < f64::EPSILON);
    }

    #[cfg(feature = "flac")]
    #[test]
    fn test_extract_no_cuesheet() {
        let flac_data = make_test_flac(44100, None);
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("nocue.flac");
        std::fs::write(&path, &flac_data).expect("write FLAC");

        let result = extract_embedded_cue_from_flac(&path).expect("extract should succeed");

        assert!(
            result.is_none(),
            "FLAC without CUESHEET must return None, not error"
        );
    }

    #[cfg(feature = "flac")]
    #[test]
    fn test_extract_flac_timings() {
        let sample_rate: u32 = 44100;

        // Build a CUESHEET that exercises sample→seconds conversion and INDEX 00/01.
        let flac_cue = flac::metadata::CueSheet {
            media_catalog_number: "TEST123".into(),
            lead_in: 8820, // 0.2 s of lead-in (ignored by our converter)
            is_cd: false,
            tracks: vec![
                // Track 1: starts at 0 with INDEX 01 only
                flac::metadata::CueSheetTrack {
                    offset: 0,
                    number: 1,
                    isrc: String::new(),
                    is_audio: true,
                    is_pre_emphasis: false,
                    indices: vec![flac::metadata::CueSheetTrackIndex {
                        offset: 0,
                        number: 1,
                    }],
                },
                // Track 2: starts at 44100 (1.0 s), with INDEX 00 (pregap) and INDEX 01
                flac::metadata::CueSheetTrack {
                    offset: 44100,
                    number: 2,
                    isrc: String::new(),
                    is_audio: true,
                    is_pre_emphasis: false,
                    indices: vec![
                        flac::metadata::CueSheetTrackIndex {
                            offset: 0, // INDEX 00 at track start = 44100 total
                            number: 0,
                        },
                        flac::metadata::CueSheetTrackIndex {
                            offset: 4410, // INDEX 01 0.1 s later = 48510 total
                            number: 1,
                        },
                    ],
                },
                // Track 3: starts at 88200 (2.0 s), INDEX 01 only
                flac::metadata::CueSheetTrack {
                    offset: 88200,
                    number: 3,
                    isrc: String::new(),
                    is_audio: true,
                    is_pre_emphasis: false,
                    indices: vec![flac::metadata::CueSheetTrackIndex {
                        offset: 0,
                        number: 1,
                    }],
                },
            ],
        };

        let flac_data = make_test_flac(sample_rate, Some(&flac_cue));
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("timings.flac");
        std::fs::write(&path, &flac_data).expect("write FLAC");

        let sheet = extract_embedded_cue_from_flac(&path)
            .expect("extract should succeed")
            .expect("should have CUESHEET");

        assert_eq!(sheet.tracks.len(), 3, "expected 3 tracks");

        // Track 1: absolute 0 → 0.0 s, no pregap
        let t1 = &sheet.tracks[0];
        assert_eq!(t1.track_number, "01");
        assert!((t1.index_start - 0.0).abs() < f64::EPSILON);
        assert!(t1.index_pregap.is_none());

        // Track 2: pregap at 44100/44100=1.0 s, start at 48510/44100=1.1 s
        let t2 = &sheet.tracks[1];
        assert_eq!(t2.track_number, "02");
        assert!(
            (t2.index_start - 1.1).abs() < 1e-9,
            "expected 1.1 s, got {}",
            t2.index_start
        );
        assert!(t2.index_pregap.is_some());
        assert!(
            (t2.index_pregap.unwrap() - 1.0).abs() < 1e-9,
            "expected pregap 1.0 s, got {}",
            t2.index_pregap.unwrap()
        );

        // Track 3: absolute 88200 → 2.0 s
        let t3 = &sheet.tracks[2];
        assert_eq!(t3.track_number, "03");
        assert!(
            (t3.index_start - 2.0).abs() < f64::EPSILON,
            "expected 2.0 s, got {}",
            t3.index_start
        );
    }

    #[cfg(feature = "flac")]
    #[test]
    fn test_extract_invalid_path() {
        let result = extract_embedded_cue_from_flac(Path::new("nonexistent_file_9999.flac"));
        assert!(result.is_err(), "Non-existent FLAC must produce an error");
    }
}
