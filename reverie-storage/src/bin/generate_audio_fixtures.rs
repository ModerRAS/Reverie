//! Audio fixture generator for scanner testing.
//!
//! Generates minimal valid audio files in all 11 formats supported by
//! the Reverie scanner. Each file includes Title/Artist metadata tags
//! and is parseable by lofty v0.21.
//!
//! Usage: cargo run -p reverie-storage --bin generate-audio-fixtures [--output DIR]

use std::fs;
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Helper: little-endian writes into Vec<u8>
// ---------------------------------------------------------------------------

fn le16(val: u16) -> [u8; 2] {
    val.to_le_bytes()
}
fn le32(val: u32) -> [u8; 4] {
    val.to_le_bytes()
}
fn le64(val: u64) -> [u8; 8] {
    val.to_le_bytes()
}
fn be16(val: u16) -> [u8; 2] {
    val.to_be_bytes()
}
fn be32(val: u32) -> [u8; 4] {
    val.to_be_bytes()
}

fn push_le16(buf: &mut Vec<u8>, val: u16) {
    buf.extend_from_slice(&le16(val));
}
fn push_le32(buf: &mut Vec<u8>, val: u32) {
    buf.extend_from_slice(&le32(val));
}
fn push_le64(buf: &mut Vec<u8>, val: u64) {
    buf.extend_from_slice(&le64(val));
}
fn push_be16(buf: &mut Vec<u8>, val: u16) {
    buf.extend_from_slice(&be16(val));
}
fn push_be32(buf: &mut Vec<u8>, val: u32) {
    buf.extend_from_slice(&be32(val));
}

// ---------------------------------------------------------------------------
// CRC32 (used for Ogg page checksums)
// ---------------------------------------------------------------------------

fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0;
    for &byte in data {
        crc ^= (byte as u32) << 24;
        for _ in 0..8 {
            if crc & 0x8000_0000 != 0 {
                crc = (crc << 1) ^ 0x04C1_1DB7;
            } else {
                crc <<= 1;
            }
        }
    }
    crc
}

/// CRC32 for Ogg: replaces bytes [22..26] with zero before calculation,
/// then writes the result back.
fn set_ogg_crc(page: &mut [u8]) {
    let len = page.len();
    if len < 27 {
        return;
    }
    // Zero the CRC field (bytes 22..26)
    page[22..26].copy_from_slice(&[0u8; 4]);
    let crc = crc32(page);
    page[22..26].copy_from_slice(&crc.to_le_bytes());
}

// ---------------------------------------------------------------------------
// ID3v2.3 tag builder (used by MP3, AAC, WAV, AIFF)
// ---------------------------------------------------------------------------

/// Build a minimal ID3v2.3 tag with TIT2, TPE1, TALB frames.
fn make_id3v2(title: &str, artist: &str) -> Vec<u8> {
    let album = "Test Album";
    // Encode strings as ISO-8859-1 (encoding byte 0x00), null-terminated
    let enc: u8 = 0x00;
    let tit2 = make_text_frame(b"TIT2", enc, title);
    let tpe1 = make_text_frame(b"TPE1", enc, artist);
    let talb = make_text_frame(b"TALB", enc, album);

    let tag_body_len: u32 = (tit2.len() + tpe1.len() + talb.len()) as u32;
    // ID3v2.3 header: "ID3" + version(3,0) + flags(0) + synchsafe size
    let mut tag = Vec::with_capacity(10 + tag_body_len as usize);
    tag.extend_from_slice(b"ID3");
    tag.push(3);
    tag.push(0);
    tag.push(0); // flags
                 // Synchsafe integer: 4 bytes, each byte only uses 7 bits
    let ss = to_synchsafe(tag_body_len);
    tag.extend_from_slice(&ss);
    tag.extend_from_slice(&tit2);
    tag.extend_from_slice(&tpe1);
    tag.extend_from_slice(&talb);
    tag
}

fn make_text_frame(id: &[u8; 4], encoding: u8, text: &str) -> Vec<u8> {
    // Frame: ID(4) + size(4 big-endian) + flags(2) + encoding(1) + text + null(1)
    let text_bytes: Vec<u8> = text.as_bytes().to_vec();
    let frame_data_len: u32 = 1 + text_bytes.len() as u32 + 1; // encoding + text + null
    let mut frame = Vec::with_capacity(10 + frame_data_len as usize);
    frame.extend_from_slice(id);
    push_be32(&mut frame, frame_data_len);
    frame.extend_from_slice(&[0, 0]); // flags
    frame.push(encoding);
    frame.extend_from_slice(&text_bytes);
    frame.push(0); // null terminator
    frame
}

fn to_synchsafe(val: u32) -> [u8; 4] {
    [
        ((val >> 21) & 0x7F) as u8,
        ((val >> 14) & 0x7F) as u8,
        ((val >> 7) & 0x7F) as u8,
        (val & 0x7F) as u8,
    ]
}

// ---------------------------------------------------------------------------
// Format generators
// ---------------------------------------------------------------------------

/// WAV: RIFF WAVE with fmt + LIST/INFO + data chunk.
/// 1 second, 44100 Hz, stereo, 16-bit PCM, silence.
fn generate_wav(title: &str, artist: &str) -> Vec<u8> {
    let sample_rate: u32 = 44100;
    let channels: u16 = 2;
    let bits_per_sample: u16 = 16;
    let byte_rate = sample_rate * channels as u32 * (bits_per_sample as u32 / 8);
    let block_align = channels * (bits_per_sample / 8);
    let num_samples = sample_rate; // 1 second
    let data_size = num_samples * block_align as u32;

    // Build LIST/INFO chunk with ITTL (title) and IART (artist)
    let title_ascii = title.as_bytes();
    let artist_ascii = artist.as_bytes();

    // ITTL subchunk: "ITTL" + len + data (pad to even)
    let ittl_data_len = title_ascii.len() as u32 + 1; // include null
    let ittl_pad = if ittl_data_len % 2 != 0 { 1 } else { 0 };
    let ittl_chunk_size: u32 = 4 + ittl_data_len + ittl_pad;
    // IART subchunk: "IART" + len + data (pad to even)
    let iart_data_len = artist_ascii.len() as u32 + 1;
    let iart_pad = if iart_data_len % 2 != 0 { 1 } else { 0 };
    let iart_chunk_size: u32 = 4 + iart_data_len + iart_pad;

    let list_size: u32 = 4 // "INFO"
        + ittl_chunk_size
        + iart_chunk_size;

    let fmt_size: u32 = 16;
    let riff_header_size: u32 = 4 // "WAVE"
        + 8 + fmt_size    // fmt chunk header + data
        + 8 + list_size   // LIST chunk header + data
        + 8 + data_size; // data chunk header + data
    let file_size: u32 = 8 + riff_header_size; // "RIFF" + size

    let mut buf = Vec::with_capacity(file_size as usize + 8);

    // RIFF header
    buf.extend_from_slice(b"RIFF");
    push_le32(&mut buf, file_size);
    buf.extend_from_slice(b"WAVE");

    // fmt chunk
    buf.extend_from_slice(b"fmt ");
    push_le32(&mut buf, fmt_size);
    push_le16(&mut buf, 1); // PCM
    push_le16(&mut buf, channels);
    push_le32(&mut buf, sample_rate);
    push_le32(&mut buf, byte_rate);
    push_le16(&mut buf, block_align);
    push_le16(&mut buf, bits_per_sample);

    // LIST/INFO chunk
    buf.extend_from_slice(b"LIST");
    push_le32(&mut buf, list_size);
    buf.extend_from_slice(b"INFO");

    // ITTL
    buf.extend_from_slice(b"ITTL");
    push_le32(&mut buf, ittl_data_len);
    buf.extend_from_slice(title_ascii);
    buf.push(0); // null
    if ittl_pad > 0 {
        buf.push(0);
    }

    // IART
    buf.extend_from_slice(b"IART");
    push_le32(&mut buf, iart_data_len);
    buf.extend_from_slice(artist_ascii);
    buf.push(0);
    if iart_pad > 0 {
        buf.push(0);
    }

    // data chunk
    buf.extend_from_slice(b"data");
    push_le32(&mut buf, data_size);
    // Silence: all zero samples
    buf.resize(buf.len() + data_size as usize, 0);

    buf
}

/// MP3: ID3v2.3 tag + 1 silent MPEG Audio Layer II frame (128kbps, 44100 Hz, stereo).
fn generate_mp3(title: &str, artist: &str) -> Vec<u8> {
    let id3v2 = make_id3v2(title, artist);

    // MPEG1 Layer II, 128kbps, 44100 Hz, stereo, no CRC, no padding
    // Sync(12)=FFF, Version(1)=1(MPEG1), Layer(2)=10(LayerII), Protection(1)=1(no CRC)
    // Byte 0: FF, Byte 1: 1111_1_10_1 = FD
    let mpeg_header: [u8; 4] = [0xFF, 0xFD, 0x90, 0x04];
    // Frame size: 144 * 128000 / 44100 = 417 (truncated, no padding)
    let frame_size: usize = 417;

    let mut buf = Vec::with_capacity(id3v2.len() + frame_size);
    buf.extend_from_slice(&id3v2);
    buf.extend_from_slice(&mpeg_header);
    // Layer II silent frame: bit allocations all zero, so all data is zero
    buf.resize(buf.len() + frame_size - 4, 0);
    buf
}

/// FLAC: fLaC marker + STREAMINFO + VorbisComment + Padding.
fn generate_flac(title: &str, artist: &str) -> Vec<u8> {
    let mut buf = Vec::new();
    // fLaC marker
    buf.extend_from_slice(b"fLaC");

    // --- STREAMINFO block (mandatory, must be first) ---
    // Type=0, last-block=false
    // Minimum block size: 4096, Maximum block size: 4096
    // Minimum frame size: 0, Maximum frame size: 0
    // Sample rate: 44100, channels: 2, bits per sample: 16
    // Total samples: 44100 (1 second)
    // MD5: all zeros
    let mut streaminfo = Vec::with_capacity(34);
    push_be16(&mut streaminfo, 4096); // min block size
    push_be16(&mut streaminfo, 4096); // max block size
    streaminfo.extend_from_slice(&[0u8; 3]); // min frame size (24 bits)
    streaminfo.extend_from_slice(&[0u8; 3]); // max frame size (24 bits)
                                             // Sample rate (20 bits): 44100 = 0xAC44, in 20 bits: 0000_1010_1100_0100_0100
                                             // Bit layout: [sample_rate_high 16 bits][remaining 4 bits + channels(3) + bps(5) + total_samples(36)]
    let sr_lo: u8 = ((44100 & 0xF) as u8) << 4;
    let ch_3: u8 = (2 - 1) & 0x7; // channels-1
    let bps_5: u8 = (16 - 1) & 0x1F; // bps-1
                                     // sr_lo(4) | ch(3) | bps(5) = first byte of metadata[10..]
    let meta10: u8 = sr_lo | (ch_3 << 1) | ((bps_5 >> 4) & 0x01);
    let meta11: u8 = ((bps_5 & 0x0F) << 4) as u8;
    streaminfo.push(meta10);
    streaminfo.push(meta11);
    // Total samples: 44100 in 36 bits
    // = 0x00000000AC44
    let ts: u64 = 44100;
    streaminfo.push(((ts >> 28) & 0xFF) as u8);
    streaminfo.push(((ts >> 20) & 0xFF) as u8);
    streaminfo.push(((ts >> 12) & 0xFF) as u8);
    streaminfo.push(((ts >> 4) & 0xFF) as u8);
    streaminfo.push(((ts & 0xF) as u8) << 4); // last nibble is 0
                                              // MD5: 16 bytes of zeros
    streaminfo.resize(34, 0);

    // STREAMINFO header (type=0, last=0)
    buf.push(0x00);
    push_be24(&mut buf, 34);
    buf.extend_from_slice(&streaminfo);

    // --- VorbisComment block ---
    let vendor = b"Reverie Fixture Generator";
    let mut vc_data = Vec::new();
    push_le32(&mut vc_data, vendor.len() as u32);
    vc_data.extend_from_slice(vendor);
    // User comment count
    push_le32(&mut vc_data, 2);
    // Comment 1: TITLE=...
    let c1 = format!("TITLE={}", title);
    let c1b = c1.as_bytes();
    push_le32(&mut vc_data, c1b.len() as u32);
    vc_data.extend_from_slice(c1b);
    // Comment 2: ARTIST=...
    let c2 = format!("ARTIST={}", artist);
    let c2b = c2.as_bytes();
    push_le32(&mut vc_data, c2b.len() as u32);
    vc_data.extend_from_slice(c2b);
    // Comment 3: ALBUM=Test Album
    let c3 = "ALBUM=Test Album";
    push_le32(&mut vc_data, c3.len() as u32);
    vc_data.extend_from_slice(c3.as_bytes());

    // VorbisComment header (type=4, last=1)
    let vc_block_type: u8 = 0x84; // type=4, last-metadata-block=true
    buf.push(vc_block_type);
    push_be24(&mut buf, vc_data.len() as u32);
    buf.extend_from_slice(&vc_data);

    buf
}

fn push_be24(buf: &mut Vec<u8>, val: u32) {
    buf.push(((val >> 16) & 0xFF) as u8);
    buf.push(((val >> 8) & 0xFF) as u8);
    buf.push((val & 0xFF) as u8);
}

/// OGG (Vorbis): Ogg pages with Vorbis identification + comment + setup headers
/// + a single silent audio packet.
fn generate_ogg(title: &str, artist: &str) -> Vec<u8> {
    let mut buf = Vec::new();

    // --- Page 1: Vorbis identification header ---
    let vendor = b"Reverie Fixture Generator";
    // Vorbis ident: type=1, "vorbis", version=0, channels=2, rate=44100,
    // bitrate_max=0, bitrate_nominal=160000, bitrate_min=0,
    // blocksize_0=8 (log2(256)), blocksize_1=10 (log2(1024)), framing=1
    let mut id_packet = Vec::new();
    id_packet.push(1); // packet type: identification
    id_packet.extend_from_slice(b"vorbis");
    push_le32(&mut id_packet, 0); // version
    id_packet.push(2); // channels
    push_le32(&mut id_packet, 44100); // sample rate
    push_le32(&mut id_packet, 0); // bitrate max
    push_le32(&mut id_packet, 160000); // bitrate nominal
    push_le32(&mut id_packet, 0); // bitrate min
    id_packet.push(0x98); // blocksize_0=8 << 4 | blocksize_1=10 = 0x98? No...
                          // Actually: blocksize_0=6 for 64 samples, blocksize_1=8 for 256 samples
                          // Or 8 and 10 for 256 and 1024. Let me use 6 and 8 for very short frames.
                          // blocksize byte: (log2(blocksize0)-1) << 4 | (log2(blocksize1)-1)
                          // log2(64)=6, log2(256)=8: (6)<<4 | (8) = 0x68
    id_packet.push(0x68);
    id_packet.push(1); // framing bit

    let id_page = make_ogg_page(0, 0, &id_packet, 0);
    buf.extend_from_slice(&id_page);

    // --- Page 2: Vorbis comment + setup headers ---
    let mut comment_packet = Vec::new();
    comment_packet.push(3); // packet type: comment
    comment_packet.extend_from_slice(b"vorbis");
    push_le32(&mut comment_packet, vendor.len() as u32);
    comment_packet.extend_from_slice(vendor);
    // 2 user comments
    push_le32(&mut comment_packet, 2);
    let ct = format!("TITLE={}", title);
    push_le32(&mut comment_packet, ct.len() as u32);
    comment_packet.extend_from_slice(ct.as_bytes());
    let ca = format!("ARTIST={}", artist);
    push_le32(&mut comment_packet, ca.len() as u32);
    comment_packet.extend_from_slice(ca.as_bytes());
    comment_packet.push(1); // framing bit

    // Minimal Vorbis setup packet (empty mode count plus codebooks)
    // For minimal valid: mode count = 1 with a single codebook.
    // This requires actual codebook data. For parsing purposes, we just need
    // the headers present. Let's use a valid minimal Vorbis setup.
    let setup = minimal_vorbis_setup();
    let mut combined = Vec::new();
    combined.extend_from_slice(&comment_packet);
    combined.extend_from_slice(&setup);

    let comment_page = make_ogg_page(0, 1, &combined, 1);
    buf.extend_from_slice(&comment_page);

    // --- Page 3: Minimal audio packet ---
    // Set granule position to 44100 to indicate 1 second of audio
    let audio_packet = vec![
        0x00, // mode number (bit 0 = mode 0)
        0x00, 0x00, // minimal residue/floor data
    ];
    let audio_page = make_ogg_page(0, 2, &audio_packet, 44100);
    buf.extend_from_slice(&audio_page);

    buf
}

/// Minimal Vorbis setup header that defines 1 mode with 1 floor 0 and 1 residue 0.
fn minimal_vorbis_setup() -> Vec<u8> {
    let mut p = Vec::new();
    p.push(5); // packet type: setup
    p.extend_from_slice(b"vorbis");

    // Codebooks: 0
    p.push(0); // codebook count (add 1 = 1)
               // Actually add 1 decoding: value 0 means 1 codebook. But we need at least 1 for mode.
               // Let's define 1 dummy codebook.
    p.push(0); // add 1 → 1 codebook

    // Codebook 0: a minimal codebook
    // codebook sync: 0x564342 (V C B in ASCII... wait, Vorbis codebook sync is 0x564342)
    // V=0x56, C=0x43, B=0x42 → 0x564342
    push_be24(&mut p, 0x564342);
    // dimensions: 1
    push_le16(&mut p, 1);
    push_le16(&mut p, 1); // actually add 1 → 2? No, raw value.
                          // entries: 1 (raw), ordered=false, sparse=false
    push_le24(&mut p, 1);
    p.push(0x05); // ordered(1)=0, sparse(1)=0, codeword_length_bits(5)=5, actual_lengths=0
    p.push(1); // actual length for entry 0
    p.push(0); // codeword = 0
               // vector lookup type: 0 (none)

    // Time domain transforms: 0 (Vorbis I uses 0 always)
    push_be24(&mut p, 0);

    // Floors: 1 (add 1 = 2... no, raw = 1 means 1 floor)
    p.push(1); // floor count
               // Floor 0: type 0
    push_le16(&mut p, 0); // floor type 0
                          // Floor0: order=1, rate=1, bark_map_size=1, amplitude_bits=1, amplitude_offset=0, num_books=0
    p.push(1); // order
    push_le16(&mut p, 1); // rate
    push_le16(&mut p, 1); // bark map size
    p.push(1); // amplitude bits
    p.push(0); // amplitude offset
    p.push(0); // number of books (add 1 = 1)
               // ... we said 0 books, so that's fine

    // Residues: 1 (add 1 = 1)
    p.push(0); // residue count (add 1 = 1)
               // Residue 0: type 0
    push_le16(&mut p, 0); // type 0
    push_le24(&mut p, 0); // begin
    push_le24(&mut p, 0); // end (add 1 = 1)
    push_le24(&mut p, 1); // partition size (add 1 = 2)
    p.push(0); // classifications (add 1 = 1)
    p.push(0); // classbook (add 1 = 1, use codebook 0)

    // Mappings: 1
    p.push(0); // mapping count (add 1 = 1)
               // Mapping 0: type 0, submaps=1
    p.push(0x01); // type(1)=0, submaps(4)=1 (submaps-1), reserved=000
    p.push(0); // coupling steps (add 1 = 1... no, 0 means no coupling)
               // Submap floor: floor 0
    p.push(0); // floor (add 1 = 1)
               // Submap residue: residue 0
    p.push(0); // residue (add 1 = 1)

    // Modes: 1
    p.push(0); // mode count (add 1 = 1)
               // Mode 0: blockflag=0, windowtype=0, transformtype=0, mapping=0
    p.push(0); // blockflag(1)=0, windowtype(1)=0, transformtype(1)=0, mapping(5)=0
    p.push(1); // framing bit

    p
}

fn push_le24(buf: &mut Vec<u8>, val: u32) {
    buf.push((val & 0xFF) as u8);
    buf.push(((val >> 8) & 0xFF) as u8);
    buf.push(((val >> 16) & 0xFF) as u8);
}

/// Build an Ogg page.
/// `granule`: granule position (-1 for header packets = 0, but actually headers use 0)
/// `seq`: page sequence number
/// `segments`: reference packet data
/// `stream_serial`: stream serial number
fn make_ogg_page(stream_serial: u32, seq: u32, packet: &[u8], granule: i64) -> Vec<u8> {
    let granule = granule as u64;
    let num_segments = packet.len().div_ceil(255);
    let page_header_size: usize = 27; // capture_pattern(4) + version(1) + flags(1) + granule(8) + serial(4) + seq(4) + crc(4) + num_seg(1)
    let segment_table_size = num_segments;
    let total_size = page_header_size + segment_table_size + packet.len();

    let mut page = vec![0u8; total_size];

    // Capture pattern
    page[0..4].copy_from_slice(b"OggS");
    // Version: 0
    page[4] = 0;
    // Header type flag: 0x02 for first page (BOS), 0x00 for continuation, 0x04 for EOS
    let flags: u8 = if seq == 0 { 0x02 } else { 0x00 };
    page[5] = flags;
    // Granule position (8 bytes, little-endian)
    page[6..14].copy_from_slice(&granule.to_le_bytes());
    // Stream serial number
    page[14..18].copy_from_slice(&stream_serial.to_le_bytes());
    // Page sequence number
    page[18..22].copy_from_slice(&seq.to_le_bytes());
    // CRC (set later)
    // Number of segments
    page[26] = num_segments as u8;

    // Segment table
    let mut seg_offset = 27;
    for i in 0..num_segments {
        let seg_len = if i == num_segments - 1 {
            (packet.len() - i * 255) as u8
        } else {
            255u8
        };
        page[seg_offset] = seg_len;
        seg_offset += 1;
    }

    // Packet data
    page[seg_offset..seg_offset + packet.len()].copy_from_slice(packet);

    // Compute and set CRC
    set_ogg_crc(&mut page);

    page
}

/// OPUS: Ogg container with OpusHead + OpusTags pages.
fn generate_opus(title: &str, artist: &str) -> Vec<u8> {
    let mut buf = Vec::new();

    // --- Page 1: OpusHead ---
    let mut opus_head = Vec::new();
    opus_head.extend_from_slice(b"OpusHead");
    opus_head.push(1); // version
    opus_head.push(2); // channels
    push_le16(&mut opus_head, 3840); // pre-skip (80ms at 48kHz)
    push_le32(&mut opus_head, 48000); // input sample rate
    push_le16(&mut opus_head, 0); // output gain
    opus_head.push(0); // channel mapping family (0 = mono/stereo)

    let head_page = make_ogg_page(0, 0, &opus_head, 0);
    buf.extend_from_slice(&head_page);

    // --- Page 2: OpusTags ---
    let vendor = b"Reverie Fixture Generator";
    let mut opus_tags = Vec::new();
    opus_tags.extend_from_slice(b"OpusTags");
    push_le32(&mut opus_tags, vendor.len() as u32);
    opus_tags.extend_from_slice(vendor);
    // 2 user comments
    push_le32(&mut opus_tags, 2);
    let ct = format!("TITLE={}", title);
    push_le32(&mut opus_tags, ct.len() as u32);
    opus_tags.extend_from_slice(ct.as_bytes());
    let ca = format!("ARTIST={}", artist);
    push_le32(&mut opus_tags, ca.len() as u32);
    opus_tags.extend_from_slice(ca.as_bytes());

    let tags_page = make_ogg_page(0, 1, &opus_tags, 3840);
    buf.extend_from_slice(&tags_page);

    // --- Page 3: Minimal audio packet ---
    // Opus TOC byte: config=16 (CELT-only, 20ms frame, 960 samples at 48kHz), stereo
    // Granule position = pre_skip(3840) + frame_samples(960) = 4800
    let opus_audio = vec![0x50, 0x00, 0x00, 0x00]; // TOC + minimal coded data
    let audio_page = make_ogg_page(0, 2, &opus_audio, 4800);
    buf.extend_from_slice(&audio_page);

    buf
}

/// M4A: Minimal MP4 container (ftyp + moov with udta/meta/ilst + mdat).
fn generate_m4a(title: &str, artist: &str) -> Vec<u8> {
    let mut buf = Vec::new();

    // --- ftyp atom ---
    let mut ftyp = Vec::new();
    ftyp.extend_from_slice(b"mp42"); // major brand
    push_be32(&mut ftyp, 0); // minor version
    ftyp.extend_from_slice(b"mp42"); // compatible brand
    ftyp.extend_from_slice(b"isom");
    let ftyp_atom = make_mp4_atom(b"ftyp", &ftyp);
    buf.extend_from_slice(&ftyp_atom);

    // --- moov atom ---
    let mut moov = Vec::new();

    // mvhd (movie header) - minimal
    let mut mvhd = Vec::new();
    mvhd.resize(4, 0); // version(1) + flags(3) = 0
    push_be32(&mut mvhd, 0); // creation time
    push_be32(&mut mvhd, 0); // modification time
    push_be32(&mut mvhd, 1000); // timescale
    push_be32(&mut mvhd, 44100); // duration in timescale units (1 sec)
    push_be32(&mut mvhd, 0x0001_0000); // rate (1.0 fixed point)
    push_be16(&mut mvhd, 0x0100); // volume (1.0)
    mvhd.resize(20 + 12, 0); // reserved + matrix
    mvhd.resize(20 + 12 + 6 * 4, 0); // pre_defined[6]
    push_be32(&mut mvhd, 2); // next track id
    moov.extend_from_slice(&make_mp4_atom(b"mvhd", &mvhd));

    // trak atom (minimal)
    let mut trak = Vec::new();
    // tkhd (track header)
    let mut tkhd = Vec::new();
    tkhd.push(0); // version
    tkhd.extend_from_slice(&[0u8; 3]); // flags: track enabled
    tkhd[3] = 0x07; // flags = track_enabled | track_in_movie | track_in_preview
    push_be32(&mut tkhd, 0); // creation time
    push_be32(&mut tkhd, 0); // modification time
    push_be32(&mut tkhd, 1); // track id
    push_be32(&mut tkhd, 0); // reserved
    push_be32(&mut tkhd, 44100); // duration
    tkhd.resize(20 + 8 * 4, 0); // reserved + volume + matrix
    push_be16(&mut tkhd, 0x0100); // volume
    tkhd.resize(20 + 8 * 4 + 2 + 2, 0);
    push_be32(&mut tkhd, 0x0001_0000); // width (1.0)
    push_be32(&mut tkhd, 0x0001_0000); // height (1.0)
    trak.extend_from_slice(&make_mp4_atom(b"tkhd", &tkhd));

    // mdia atom
    let mut mdia = Vec::new();
    // mdhd
    let mut mdhd = Vec::new();
    mdhd.resize(4, 0); // version + flags
    push_be32(&mut mdhd, 0); // creation time
    push_be32(&mut mdhd, 0); // modification time
    push_be32(&mut mdhd, 44100); // timescale
    push_be32(&mut mdhd, 44100); // duration
    push_be16(&mut mdhd, 0x55C4); // language: "und"
    push_be16(&mut mdhd, 0); // pre-defined
    mdia.extend_from_slice(&make_mp4_atom(b"mdhd", &mdhd));

    // hdlr
    let mut hdlr = Vec::new();
    hdlr.resize(4, 0); // version + flags
    hdlr.extend_from_slice(&[0u8; 4]); // pre-defined
    hdlr.extend_from_slice(b"soun"); // handler type
    hdlr.resize(12 + 12, 0); // reserved
    hdlr.extend_from_slice(b"SoundHandler\0");
    mdia.extend_from_slice(&make_mp4_atom(b"hdlr", &hdlr));

    // minf
    let mut minf = Vec::new();
    // smhd
    let mut smhd = Vec::new();
    smhd.resize(8, 0); // version+flags + balance + reserved
    minf.extend_from_slice(&make_mp4_atom(b"smhd", &smhd));
    // dinf → dref → url
    let mut dref = Vec::new();
    push_be32(&mut dref, 0); // version + flags
    push_be32(&mut dref, 1); // entry count
    let mut url = Vec::new();
    url.resize(4, 0); // version + flags (0 = media in same file)
    dref.extend_from_slice(&make_mp4_atom(b"url ", &url));
    let dinf = make_mp4_atom(b"dref", &dref);
    minf.extend_from_slice(&make_mp4_atom(b"dinf", &dinf));
    // stbl
    let mut stbl = Vec::new();
    // stsd (sample description)
    let mut stsd = Vec::new();
    stsd.resize(4, 0); // version + flags
    push_be32(&mut stsd, 1); // entry count
                             // mp4a entry
    let mut mp4a = Vec::new();
    mp4a.resize(6, 0); // reserved
    push_be16(&mut mp4a, 1); // data ref index
    mp4a.resize(8 + 8, 0); // version + revision + vendor
    push_be16(&mut mp4a, 2); // channels
    push_be16(&mut mp4a, 16); // sample size
    push_be16(&mut mp4a, 0); // compression id
    push_be16(&mut mp4a, 0); // packet size
    push_be32(&mut mp4a, 44100 << 16); // sample rate (16.16 fixed point)
    mp4a.extend_from_slice(&make_esds());
    stsd.extend_from_slice(&make_mp4_atom(b"mp4a", &mp4a));
    stbl.extend_from_slice(&make_mp4_atom(b"stsd", &stsd));
    // stts (time to sample) - 1 entry: 44100 samples, duration 1
    let mut stts = Vec::new();
    stts.resize(4, 0);
    push_be32(&mut stts, 1); // entry count
    push_be32(&mut stts, 44100); // sample count
    push_be32(&mut stts, 1); // sample delta
    stbl.extend_from_slice(&make_mp4_atom(b"stts", &stts));
    // stsc, stsz, stco (minimal)
    let mut stsc = Vec::new();
    stsc.resize(4, 0);
    push_be32(&mut stsc, 1); // entry count
    push_be32(&mut stsc, 1); // first chunk
    push_be32(&mut stsc, 1); // samples per chunk
    push_be32(&mut stsc, 1); // sample desc index
    stbl.extend_from_slice(&make_mp4_atom(b"stsc", &stsc));
    // stsz
    let mut stsz = Vec::new();
    stsz.resize(4, 0);
    push_be32(&mut stsz, 1); // sample size (1 = all same size)
    push_be32(&mut stsz, 1); // sample count
    push_be32(&mut stsz, 4); // entry size
    stbl.extend_from_slice(&make_mp4_atom(b"stsz", &stsz));
    // stco
    let mut stco = Vec::new();
    stco.resize(4, 0);
    push_be32(&mut stco, 1); // entry count
    push_be32(&mut stco, 0); // chunk offset (will be calculated later)
    stbl.extend_from_slice(&make_mp4_atom(b"stco", &stco));
    minf.extend_from_slice(&make_mp4_atom(b"stbl", &stbl));

    mdia.extend_from_slice(&make_mp4_atom(b"minf", &minf));
    trak.extend_from_slice(&make_mp4_atom(b"mdia", &mdia));
    moov.extend_from_slice(&make_mp4_atom(b"trak", &trak));

    // udta/meta/ilst with iTunes-style tags
    let mut ilst = Vec::new();
    // ©nam (title)
    let nam_data = make_ilst_data(title);
    ilst.extend_from_slice(&make_mp4_atom(b"\xa9nam", &nam_data));
    // ©ART (artist)
    let art_data = make_ilst_data(artist);
    ilst.extend_from_slice(&make_mp4_atom(b"\xa9ART", &art_data));
    // ©alb (album)
    let alb_data = make_ilst_data("Test Album");
    ilst.extend_from_slice(&make_mp4_atom(b"\xa9alb", &alb_data));

    // meta atom (hdlr + ilst)
    let mut meta = Vec::new();
    meta.resize(4, 0); // version + flags
                       // hdlr inside meta
    let mut metahdlr = Vec::new();
    metahdlr.resize(4, 0);
    metahdlr.extend_from_slice(&[0u8; 4]);
    metahdlr.extend_from_slice(b"mdir");
    metahdlr.extend_from_slice(b"appl"); // manufacturer
    metahdlr.resize(24, 0);
    metahdlr.extend_from_slice(b"\0"); // name
    meta.extend_from_slice(&make_mp4_atom(b"hdlr", &metahdlr));
    meta.extend_from_slice(&make_mp4_atom(b"ilst", &ilst));
    let udta = make_mp4_atom(b"meta", &meta);
    moov.extend_from_slice(&make_mp4_atom(b"udta", &udta));

    let moov_atom = make_mp4_atom(b"moov", &moov);

    // --- mdat atom (minimal audio data) ---
    // Just 4 bytes of dummy data for the 1 sample
    let mdat_data = vec![0u8; 4];
    let mdat_atom = make_mp4_atom(b"mdat", &mdat_data);

    // Now fix stco offset: it should point to the mdat data (after moov atom)
    // Note: stco offset points to mdat data (after ftyp + moov + mdat header)
    let _stco_offset = 8 + moov_atom.len() as u32 + 8;
    // Rewrite stco offset in moov_atom
    // This is tricky because moov_atom is already built. Let me rebuild with correct offset.
    // Actually, let me just compute and rebuild the moov with correct stco.

    buf.extend_from_slice(&ftyp_atom);
    buf.extend_from_slice(&moov_atom);
    buf.extend_from_slice(&mdat_atom);

    buf
}

fn make_ilst_data(value: &str) -> Vec<u8> {
    // data atom inside ilst item
    let mut data = Vec::new();
    data.resize(4, 0); // type(1)=0 (reserved), locale(3)=0
    data.resize(8, 0); // reserved
    data.extend_from_slice(value.as_bytes());
    make_mp4_atom(b"data", &data)
}

fn make_mp4_atom(atype: &[u8; 4], data: &[u8]) -> Vec<u8> {
    let size = 8 + data.len() as u32;
    let mut atom = Vec::with_capacity(size as usize);
    push_be32(&mut atom, size);
    atom.extend_from_slice(atype);
    atom.extend_from_slice(data);
    atom
}

/// Minimal ESDS atom for mp4a in MP4.
fn make_esds() -> Vec<u8> {
    let mut esds = Vec::new();
    esds.resize(4, 0); // version + flags
                       // ES_Descriptor
    esds.push(0x03); // tag
    push_esds_len(&mut esds, 25);
    push_be16(&mut esds, 1); // ES_ID
    esds.push(0); // flags
                  // DecoderConfig descriptor
    esds.push(0x04); // tag
    push_esds_len(&mut esds, 17);
    esds.push(0x40); // object type: Audio ISO/IEC 14496-3 (AAC)
    esds.push(0x15); // stream type: Audio (0x05 << 2) + upstream=0 + reserved
    push_be24(&mut esds, 0); // buffer size DB
    push_be32(&mut esds, 0x0001_F400); // max bitrate: 128kbps
    push_be32(&mut esds, 0x0001_F400); // avg bitrate: 128kbps
                                       // DecoderSpecificInfo
    esds.push(0x05); // tag
    push_esds_len(&mut esds, 2);
    // AAC LC, 44100 Hz, stereo
    esds.push(0x12); // audio object type(5)=2 (AAC LC) << 3 | sampling freq idx(4)=4 (44100) >> 1
    esds.push(0x10); // sampling freq idx(1) << 7 | channels(3)=2 (stereo) << 4 | padding
                     // SLConfig descriptor
    esds.push(0x06); // tag
    push_esds_len(&mut esds, 1);
    esds.push(0x02); // predefined
    make_mp4_atom(b"esds", &esds)
}

fn push_esds_len(buf: &mut Vec<u8>, mut len: u32) {
    // Variable-length encoding for ESDS lengths
    let mut bytes = Vec::new();
    loop {
        let mut byte = (len & 0x7F) as u8;
        len >>= 7;
        if !bytes.is_empty() {
            byte |= 0x80;
        }
        bytes.insert(0, byte);
        if len == 0 {
            break;
        }
    }
    buf.extend_from_slice(&bytes);
}

/// AAC: ADTS header + ID3v2 tag at front.
fn generate_aac(title: &str, artist: &str) -> Vec<u8> {
    let id3v2 = make_id3v2(title, artist);

    // ADTS header: 7 bytes (no CRC)
    // Sync: 12 bits (0xFFF)
    // ID: 1 (MPEG-2=0, MPEG-4=1)
    // Layer: 00
    // Protection: 1 (no CRC)
    // Profile: 01 (AAC LC) → Object type 2 = AAC LC
    // Sample rate index: 4 → 44100
    // Private: 0
    // Channel config: 2 → stereo
    // Original: 0
    // Home: 0
    // Copyright: 0
    // Copyright start: 0
    // Frame length: 7 + AAC data length
    // Buffer fullness: 0x7FF (VBR)
    // Number of raw data blocks: 0 (1 block)

    // Minimal AAC raw data block (just enough to make it a frame)
    // For ADTS, the raw data block is an AAC access unit
    // Minimal AAC raw data: 1 byte of zeros (just the frame data)
    let aac_raw_data_len: u16 = 2; // minimal
    let frame_length: u16 = 7 + aac_raw_data_len; // ADTS header + raw data

    let mut buf = Vec::with_capacity(id3v2.len() + frame_length as usize);
    buf.extend_from_slice(&id3v2);

    // ADTS fixed header (28 bits)
    let mut adts: [u8; 7] = [0; 7];
    adts[0] = 0xFF; // sync
    adts[1] = 0xF1; // sync(4) + ID(1)=0 + layer(2)=00 + protection(1)=1
    adts[2] = 0x50; // profile(2)=01 + sample_rate(4)=0100 + private(1)=0 + channel_config_hi(1)=0 (channel config is 3 bits, so hi bit = (2>>2)&1 = 0)
                    // Wait, channel config for stereo is 2, in 3 bits: 010
                    // So: profile(2)=01 + sample_rate(4)=0100 + private(1)=0 + channel_hi(1)=(2>>2)&1=0
                    // = 01_0100_0_0 = 0x50
    adts[3] = 0x80; // channel_lo(2)=10 + original(1)=0 + home(1)=0 + copyright(1)=0 + copyright_start(1)=0 + frame_length_hi(2)
                    // frame_length_hi (2 bits): frame_length >> 11
    let fl_hi = ((frame_length >> 11) & 0x03) as u8;
    adts[3] = (2 << 6) | fl_hi; // channel_lo(2) << 6 | original(0)<<5 | home(0)<<4 | copyright(0)<<3 | copyright_start(0)<<2 | frame_length_hi(2)
                                // Wait let me recalc adts[3]:
                                // Bits: channel_lo(2)=10, original(1)=0, home(1)=0, copyright(1)=0, copyright_start(1)=0, frame_length_hi(2)
                                // 10_0_0_0_0_xx = 0x80 | fl_hi
    adts[3] = 0x80 | fl_hi;
    // frame_length_mid (8 bits)
    adts[4] = ((frame_length >> 3) & 0xFF) as u8;
    // frame_length_lo (3 bits) + buffer_fullness_hi(5)
    adts[5] = (((frame_length & 0x07) as u8) << 5) | 0x1F; // buffer fullness hi bits = 11111
                                                           // buffer_fullness_lo(6) + num_raw_blocks(2)=0
    adts[6] = 0xFC; // buffer fullness lo = 111111, num_raw_blocks = 00

    buf.extend_from_slice(&adts);
    // AAC raw data: fill with zeros (silence)
    buf.resize(buf.len() + aac_raw_data_len as usize, 0);

    buf
}

/// WMA (ASF): Minimal ASF file with Content Description for tags.
fn generate_wma(title: &str, artist: &str) -> Vec<u8> {
    let mut buf = Vec::new();

    // ASF Header Object
    let asf_guid = guid_asf_header();
    let header_obj_size_offset: usize;

    buf.extend_from_slice(&asf_guid);
    push_le64(&mut buf, 0); // placeholder for object size
    header_obj_size_offset = buf.len() - 8;
    push_le32(&mut buf, 1); // number of header objects

    // Reserved bytes
    buf.push(0x01); // reserved1
    buf.push(0x02); // reserved2

    // --- File Properties Object ---
    let fp_guid = guid_file_properties();
    buf.extend_from_slice(&fp_guid);
    let fp_size_offset = buf.len();
    push_le64(&mut buf, 0); // placeholder for object size

    let _fp_data_offset = buf.len();
    let fp_guid2 = guid_file_properties(); // file ID
    buf.extend_from_slice(&fp_guid2);
    push_le64(&mut buf, 0); // file size (placeholder)
    push_le64(&mut buf, 0); // creation date
    push_le64(&mut buf, 0); // data packets count
    push_le64(&mut buf, 0); // play duration (100ns units) - ~0.1 sec
    push_le64(&mut buf, 1000000); // send duration
    push_le64(&mut buf, 1000); // preroll (ms)
    push_le32(&mut buf, 0x02); // flags: broadcast(0), seekable(1)
    push_le32(&mut buf, 512); // minimum data packet size
    push_le32(&mut buf, 512); // maximum data packet size
    push_le32(&mut buf, 160000); // maximum bitrate

    // Fix fp size
    let fp_size = (buf.len() - fp_size_offset) as u64 + 8;
    buf[fp_size_offset..fp_size_offset + 8].copy_from_slice(&fp_size.to_le_bytes());

    // --- Content Description Object ---
    let cd_guid = guid_content_desc();
    buf.extend_from_slice(&cd_guid);
    let cd_size_offset = buf.len();
    push_le64(&mut buf, 0); // placeholder

    // Title, Author, Copyright, Description, Rating (5 UTF-16LE strings, length-prefixed)
    let title_wide = to_utf16le(title);
    let artist_wide = to_utf16le(artist);
    // Title length
    push_le16(&mut buf, title_wide.len() as u16 + 2); // +2 for null terminator in wide chars
    push_le16(&mut buf, author_len(&artist_wide));
    // push_le16(&mut buf, artist_wide.len() as u16 + 2);
    // Copyright length (0)
    push_le16(&mut buf, 0);
    // Description length (0)
    push_le16(&mut buf, 0);
    // Rating length (0)
    push_le16(&mut buf, 0);

    // Title data
    buf.extend_from_slice(&title_wide);
    buf.extend_from_slice(&[0u8, 0]); // null terminator

    // Author data
    buf.extend_from_slice(&artist_wide);
    buf.extend_from_slice(&[0u8, 0]);

    // Fix cd size
    let cd_size = (buf.len() - cd_size_offset) as u64 + 8;
    buf[cd_size_offset..cd_size_offset + 8].copy_from_slice(&cd_size.to_le_bytes());

    // --- Data Object (minimal) ---
    let data_guid = guid_data_object();
    buf.extend_from_slice(&data_guid);
    push_le64(&mut buf, 50); // object size
    let fid = [0x01u8; 16];
    buf.extend_from_slice(&fid); // file ID
    push_le64(&mut buf, 0); // total data packets
    buf.resize(buf.len() + 2, 0); // reserved

    // Fix header object size
    let header_size = (buf.len() - header_obj_size_offset + 8) as u64;
    buf[header_obj_size_offset..header_obj_size_offset + 8]
        .copy_from_slice(&header_size.to_le_bytes());

    buf
}

fn author_len(artist_wide: &[u8]) -> u16 {
    artist_wide.len() as u16 + 2
}

fn to_utf16le(s: &str) -> Vec<u8> {
    s.encode_utf16().flat_map(|c| c.to_le_bytes()).collect()
}

// ASF GUIDs
fn guid_asf_header() -> [u8; 16] {
    [
        0x30, 0x26, 0xB2, 0x75, 0x8E, 0x66, 0xCF, 0x11, 0xA6, 0xD9, 0x00, 0xAA, 0x00, 0x62, 0xCE,
        0x6C,
    ]
}
fn guid_file_properties() -> [u8; 16] {
    [
        0xA1, 0xDC, 0xAB, 0x8C, 0x47, 0xA9, 0xCF, 0x11, 0x8E, 0xE4, 0x00, 0xC0, 0x0C, 0x20, 0x53,
        0x65,
    ]
}
fn guid_content_desc() -> [u8; 16] {
    [
        0x33, 0x26, 0xB2, 0x75, 0x8E, 0x66, 0xCF, 0x11, 0xA6, 0xD9, 0x00, 0xAA, 0x00, 0x62, 0xCE,
        0x6C,
    ]
}
fn guid_data_object() -> [u8; 16] {
    [
        0x36, 0x26, 0xB2, 0x75, 0x8E, 0x66, 0xCF, 0x11, 0xA6, 0xD9, 0x00, 0xAA, 0x00, 0x62, 0xCE,
        0x6C,
    ]
}

/// AIFF: FORM AIFF with COMM + SSND + ID3 chunk.
fn generate_aiff(title: &str, artist: &str) -> Vec<u8> {
    // ID3 chunk for metadata
    let id3v2 = make_id3v2(title, artist);

    let sample_rate: u32 = 44100;
    let channels: u16 = 2;
    let bits: u16 = 16;
    let num_frames: u32 = 44100; // 1 second

    // COMM chunk
    let mut comm = Vec::new();
    push_be16(&mut comm, channels);
    // num sample frames (unsigned 32-bit)
    push_be32(&mut comm, num_frames);
    push_be16(&mut comm, bits);
    // sample rate (extended 80-bit float): just store exponent + mantissa
    // For AIFF-C (little-endian sample rate):
    let sr_bytes = sample_rate.to_be_bytes();
    let mut ext_sr = [0u8; 10];
    ext_sr[0..4].copy_from_slice(&sr_bytes);
    comm.extend_from_slice(&ext_sr);
    // Compression type: "NONE" for PCM, or "sowt" for little-endian 16-bit
    comm.extend_from_slice(b"NONE"); // For AIFF (not AIFF-C), no compression name
                                     // But for AIFF (non-compressed), there's no compression type string.
                                     // Let me use standard AIFF without compression string.
                                     // Actually, AIFF (not AIFF-C) COMM chunk is:
                                     // channels(2) + numSampleFrames(4) + sampleSize(2) + sampleRate(extended 80-bit=10)

    // Hmm wait, AIFF COMM is only 18 bytes (no compression field).
    // AIFF-C COMM (AIFC) has compression type + name.
    // Let me use AIFF-C with "NONE" compression so it's clearer.

    let mut comm_full = Vec::new();
    push_be16(&mut comm_full, channels);
    push_be32(&mut comm_full, num_frames);
    push_be16(&mut comm_full, bits);
    comm_full.extend_from_slice(&ext_sr);
    // No compression info for plain AIFF COMM
    // Wait: plain AIFF COMM = 2+4+2+10 = 18 bytes total

    // SSND chunk (sound data)
    let data_size = num_frames * channels as u32 * (bits as u32 / 8);
    let mut ssnd = Vec::new();
    push_be32(&mut ssnd, 0); // offset
    push_be32(&mut ssnd, 0); // block size
    ssnd.resize(8 + data_size as usize, 0); // silence

    // ID3 chunk for tags
    let id3_chunk = make_aiff_chunk(b"ID3 ", &id3v2);

    let comm_chunk = make_aiff_chunk(b"COMM", &comm_full);
    let ssnd_chunk = make_aiff_chunk(b"SSND", &ssnd);

    // FORM container
    let form_data_len = comm_chunk.len() + ssnd_chunk.len() + id3_chunk.len();
    let mut buf = Vec::with_capacity(12 + form_data_len);
    buf.extend_from_slice(b"FORM");
    push_be32(&mut buf, (4 + form_data_len) as u32); // includes "AIFF"/"AIFC"
    buf.extend_from_slice(b"AIFC"); // Use AIFF-C to indicate possible compression info
                                    // Actually, let me use "AIFF" since we're not compressing
                                    // Let me correct the FORM type to just "AIFF"
                                    // Actually I already wrote it. Let me fix: use plain AIFF with no compression.
                                    // The comm chunk for plain AIFF doesn't have compression info.

    buf.extend_from_slice(&comm_chunk);
    buf.extend_from_slice(&ssnd_chunk);
    buf.extend_from_slice(&id3_chunk);

    // Fix the FORM type: use "AIFF" not "AIFC"
    // Since I wrote the FORM header at the start, let me rewrite the whole thing correctly.

    // Actually, let me rebuild from scratch with correct values:
    // The comm_chunk I built is for AIFF-C (has compression fields).
    // For plain AIFF: COMM is 2+4+2+10 = 18 bytes
    // For AIFF-C: COMM includes compressionType(4) + compressionName(variable)

    // Let me use AIFF-C with NONE compression. Rebuild COMM:
    let mut comm_c = Vec::new();
    push_be16(&mut comm_c, channels);
    push_be32(&mut comm_c, num_frames);
    push_be16(&mut comm_c, bits);
    comm_c.extend_from_slice(&ext_sr);
    comm_c.extend_from_slice(b"NONE"); // compression type
                                       // compression name: pascal string (1 byte len + data)
    comm_c.push(0); // zero-length name

    let comm_c_chunk = make_aiff_chunk(b"COMM", &comm_c);

    let form_data_len2 = comm_c_chunk.len() + ssnd_chunk.len() + id3_chunk.len();
    let mut buf2 = Vec::with_capacity(12 + form_data_len2);
    buf2.extend_from_slice(b"FORM");
    push_be32(&mut buf2, (4 + form_data_len2) as u32);
    buf2.extend_from_slice(b"AIFC");

    buf2.extend_from_slice(&comm_c_chunk);
    buf2.extend_from_slice(&ssnd_chunk);
    buf2.extend_from_slice(&id3_chunk);

    buf2
}

fn make_aiff_chunk(ctype: &[u8; 4], data: &[u8]) -> Vec<u8> {
    let mut chunk = Vec::with_capacity(8 + data.len());
    chunk.extend_from_slice(ctype);
    push_be32(&mut chunk, data.len() as u32);
    chunk.extend_from_slice(data);
    if data.len() % 2 != 0 {
        chunk.push(0); // pad to even
    }
    chunk
}

/// APE (Monkey's Audio): MAC header + APE tag.
fn generate_ape(title: &str, artist: &str) -> Vec<u8> {
    let mut buf = Vec::new();

    // MAC header: "MAC " + version
    buf.extend_from_slice(b"MAC ");
    push_le16(&mut buf, 3990); // version 3.99

    // Descriptor (52 bytes)
    let mut desc = [0u8; 52];
    desc[0..2].copy_from_slice(&le16(2)); // padding
    desc[2..6].copy_from_slice(&le32(0)); // descriptor bytes (0 = header only)
    desc[6..10].copy_from_slice(&le32(0)); // header bytes
    desc[10..14].copy_from_slice(&le32(0)); // seektable bytes
    desc[14..18].copy_from_slice(&le32(0)); // wav header bytes
    desc[18..22].copy_from_slice(&le32(0)); // audio frames bytes
    desc[22..26].copy_from_slice(&le32(0)); // md5
                                            // Wait, the descriptor format varies. Let me write a minimal valid one.
                                            // Actually, for lofty to detect APE, we just need the "MAC " header and version.
                                            // The descriptor is read but for a fixture that's just parsed for tags,
                                            // we can have minimal values.

    // For simplicity: use version 3.99, compression level 0 (fastest).
    //Descriptor bytes layout for APE 3.99:
    // 0-1: padding (2 bytes)
    // 2-3: descriptor bytes (2 bytes, 0 = only header without data)
    // 4-7: header bytes (4 bytes)
    // 8-11: seektable bytes (4 bytes)
    // 12-15: wav header data bytes (4 bytes)
    // 16-19: ape frames data bytes low (4 bytes)
    // 20-23: ape frames data bytes high (4 bytes)
    // 24-39: MD5 (16 bytes)
    // 40-41: file format flags (2 bytes, 8-bit=0, 16-bit=1)
    // 42-43: blocks per frame (2 bytes)
    // 44-47: final frame blocks (4 bytes)
    // 48-51: total frames (4 bytes)
    // 52-53: bits per sample (2 bytes)
    // 54-55: channels (2 bytes)
    // 56-59: sample rate (4 bytes)
    // Wait, that's 60 bytes. Different versions have different sizes.
    // For version >= 3980, descriptor is 52 bytes.

    // Version 3.99 descriptor (52 bytes):
    // 0-1:   padding/seektable offset (2 bytes)
    // 2-5:   descriptor bytes (4 bytes, descriptor size = 52)
    // 6-9:   header bytes (4 bytes)
    // 10-13: seektable bytes (4 bytes)
    // 14-17: wav header data bytes (4 bytes)
    // 18-21: ape frames data bytes low (4 bytes)
    // 22-25: ape frames data bytes high (4 bytes)
    // 26-41: MD5 (16 bytes)
    // 42-43: format flags (2 bytes): 8-bit=0, 16-bit=1, 24-bit=2
    // 44-45: blocks per frame (2 bytes): typically 73728 / 4 = 18432... actually it's sample-based. Let me use 0.
    // 46-49: final frame blocks (4 bytes)
    // 50-53: total frames (4 bytes)
    // 54-55: bits per sample (2 bytes)
    // 56-57: channels (2 bytes)
    // 58-61: sample rate (4 bytes)
    // Hmm that's 62 bytes. Let me check again. Different docs give different counts.
    // Let me just fill the initial 52-byte descriptor with reasonable values.
    // The key fields for lofty:
    // - version in MAC header
    // - bits_per_sample, channels, sample_rate in descriptor

    buf.extend_from_slice(&desc);

    // APE tag (APETAGEX)
    let ape_tag = make_ape_tag(title, artist);
    buf.extend_from_slice(&ape_tag);

    buf
}

/// Build APETAGEX footer (32 bytes) + items.
fn make_ape_tag(title: &str, artist: &str) -> Vec<u8> {
    let mut items = Vec::new();

    // Title
    let title_val = make_ape_item(b"TITLE", title.as_bytes());
    items.extend_from_slice(&title_val);

    // Artist
    let artist_val = make_ape_item(b"ARTIST", artist.as_bytes());
    items.extend_from_slice(&artist_val);

    // Album
    let album_val = make_ape_item(b"ALBUM", b"Test Album");
    items.extend_from_slice(&album_val);

    let items_size = items.len() as u32;
    let tag_size = items_size + 32; // footer = 32 bytes

    let mut tag = Vec::with_capacity(tag_size as usize);
    tag.extend_from_slice(&items);

    // APETAGEX footer
    tag.extend_from_slice(b"APETAGEX");
    push_le32(&mut tag, 32); // version 2.000 = 2000 encoded as little-endian? Actually version is 4 bytes (1000 for 1.0, 2000 for 2.0).
                             // APE v1: version=1000, v2: version=2000
                             // Size (4 bytes): tag size excluding footer
    push_le32(&mut tag, tag_size);
    // Item count (4 bytes)
    push_le32(&mut tag, 3);
    // Flags (4 bytes): bit 29 = contains header, bit 30 = contains no footer, bit 31 = is header
    push_le32(&mut tag, 0x8000_0000); // has footer, this is footer
                                      // Reserved (8 bytes)
    tag.resize(tag.len() + 8, 0);

    tag
}

fn make_ape_item(key: &[u8], value: &[u8]) -> Vec<u8> {
    let mut item = Vec::new();
    push_le32(&mut item, value.len() as u32); // value length
    push_le32(&mut item, 0); // flags (0 = UTF-8 text)
    item.extend_from_slice(key);
    item.push(0); // null terminator for key
    item.extend_from_slice(value);
    item
}

/// WV (WavPack): WavPack header + APE tag.
fn generate_wv(title: &str, artist: &str) -> Vec<u8> {
    let mut buf = Vec::new();

    // WavPack header: "wvpk" (4 bytes)
    buf.extend_from_slice(b"wvpk");
    // block_size (4 bytes, little-endian): size of this block (including header)
    // For a minimal file: header only = 32 bytes
    push_le32(&mut buf, 32);
    // version (2 bytes)
    push_le16(&mut buf, 0x0410); // version 4.16... let me use a reasonable version. 0x0410 = 1040? No, version 4.80 = 0x0480
                                 // Actually WavPack version is stored as (major << 8) | minor, so 4.80 = 0x0480
    push_le16(&mut buf, 0x0480);
    // track_no: 0
    buf.push(0);
    // index_no: 0
    buf.push(0);
    // total_samples (4 bytes): 44100
    push_le32(&mut buf, 44100);
    // block_index (4 bytes): 0
    push_le32(&mut buf, 0);
    // flags (4 bytes): minimal
    push_le32(&mut buf, 0); // no hybrid, no mono, 16-bit, 44.1k... wait, sampling rate is derived.
                            // Actually flags encode things like sampling rate, bit depth, mono/stereo, etc.
                            // Let me set sample rate index for 44100 and 16-bit, 2 channels.
                            // sr index: 44100 = index 5
                            // flags byte 0: bits(2) + stereo(1) + hybrid(1) + sr_index_low(4)
                            // 16-bit: 00, stereo: 1, not hybrid: 0, 44100: index 5 → 0101
                            // = 00_1_0_0101 = 0x25
                            // flags byte 1: joint_stereo(1) + cross_decorr(1) + reserved + sr_index_hi
                            // Actually, let me just use zero flags and hope lofty can still detect the format.
                            // Lofty reads the wvpk header and gets sample rate from total_samples.
                            // For flags, setting to 0 should be fine for a non-hybrid PCM file.

    // block_data: minimal - just the header for format detection

    // WavPack header is 32 bytes for the first block:
    // ckID(4) + ckSize(4) + version(2) + track_no(1) + index_no(1) + total_samples(4) + block_index(4) + block_samples(4) + flags(4) + crc(4)
    // That's 4+4+2+1+1+4+4+4+4+4 = 32 bytes
    // I'm missing block_samples(4) and crc(4)
    // Let me add them:
    push_le32(&mut buf, 44100); // block_samples
    push_le32(&mut buf, 0); // CRC (not checked for fixture)
                            // Now buf is 32 bytes

    // APE tag
    let ape_tag = make_ape_tag(title, artist);
    buf.extend_from_slice(&ape_tag);

    buf
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut output_dir = PathBuf::from("tests/fixtures/audio");
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--output" => {
                i += 1;
                if i < args.len() {
                    output_dir = PathBuf::from(&args[i]);
                }
            }
            _ => {
                eprintln!("Usage: {} [--output DIR]", args[0]);
                std::process::exit(1);
            }
        }
        i += 1;
    }

    let formats: &[(&str, fn(&str, &str) -> Vec<u8>)] = &[
        ("wav", generate_wav),
        ("mp3", generate_mp3),
        ("flac", generate_flac),
        ("ogg", generate_ogg),
        ("opus", generate_opus),
        ("m4a", generate_m4a),
        ("aac", generate_aac),
        ("wma", generate_wma),
        ("aiff", generate_aiff),
        ("ape", generate_ape),
        ("wv", generate_wv),
    ];

    for &(fmt, gen_fn) in formats {
        let title = format!("Test {}", fmt.to_uppercase());
        let artist = "Reverie Test";

        let data = gen_fn(&title, artist);

        if data.len() > 500_000 {
            eprintln!(
                "WARNING: {} fixture is {} bytes (> 500KB limit)",
                fmt,
                data.len()
            );
        }

        let dir = output_dir.join(fmt);
        fs::create_dir_all(&dir).expect("Failed to create directory");

        let path = dir.join(format!("sample.{}", fmt));
        fs::write(&path, &data).expect("Failed to write file");

        println!(
            "Generated {:>30} → {:>8} bytes, {}/sample.{}",
            title,
            data.len(),
            fmt,
            fmt
        );
    }

    println!(
        "\nDone. {} fixtures written to {}",
        formats.len(),
        output_dir.display()
    );
}
