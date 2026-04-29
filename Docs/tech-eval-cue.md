# CUE Support Technical Evaluation

> **Project**: Reverie music streaming server (Rust)
> **Date**: 2026-04-29
> **Scope**: CUE sheet parsing, FLAC CUESHEET metadata block reading, byte-range file reads

---

## Recommendation Summary

For CUE sheet parsing, use the **`rcue`** crate (MIT, zero-dependency, pure Rust with proven UTF-8 support) as the primary parser, with **`cue_lib`** (MIT, pure Rust, actively developed as of Apr 2026) as a forward-looking candidate once it matures. For FLAC-embedded CUESHEET metadata blocks, lofty 0.21 does NOT support them — use the **`flac`** crate (sourrust, v0.5.0) to read embedded CUESHEET data alongside lofty for VorbisComments/ID3v2 tags. For byte-range reads, OpenDAL 0.55 already fully supports `op.read_with(path).range(100..200)` on both `services-fs` and `services-memory` — no additional dependency needed.

---

## CUE Parsing Crates

### Available Crates — Summary Table

| Crate | Version | Published | Downloads | License | Stars | Pure Rust | Status |
|-------|---------|-----------|-----------|---------|-------|-----------|--------|
| [**`cue`**](https://crates.io/crates/cue) | 3.0.1 (⚠️ docs.rs broken) | 2025-05-21 | ~15,500 | GPL-2.0 | 12 | ❌ (FFI → libcue C) | 🟡 Passive maintenance; UAF security bug open |
| [**`cue-sys`**](https://crates.io/crates/cue-sys) | 2.0.1 | 2025-11-09 | ~6,800 | GPL-2.0 | 12 | ❌ (raw FFI) | 🟢 Active (tied to `cue`) |
| [**`rcue`**](https://crates.io/crates/rcue) | 0.1.3 | 2022-04-13 | ~14,000 | MIT | 4 | ✅ | 🟡 Stalled (no commits since 2022); feature-complete for reading |
| [**`cuna`**](https://crates.io/crates/cuna) | 0.7.0 | 2023-03-19 | ~21,800 | MIT | 4 | ✅ (nom-based) | 🟡 Stalled; explicit UTF-8 BOM support |
| **`cue_sheet`** | 0.3.0 | 2019-03-30 | ~5,500 | GPL-3.0 | 5 | ✅ | 🔴 **Archived** — author recommends `cue` instead |
| [**`cue_lib`**](https://codeberg.org/SuperiorOne/cue_lib) | 0.1.0 | **2026-04-17** | ~12 | MIT | — | ✅ (no_std) | 🟢 **Newest** — active development on Codeberg |

### Per-Crate Detail

#### `cue` (libcue FFI bindings)
- **Most complete** CUE command coverage — delegates to the mature `libcue` C library (67 stars, last release v2.3.0 Oct 2023, maintained as of Feb 2026).
- Supports all standard commands: `FILE`, `TRACK`, `TITLE`, `PERFORMER`, `SONGWRITER`, `INDEX`, `PREGAP`, `POSTGAP`, `ISRC`, `FLAGS`, `CATALOG`, `CDTEXTFILE`, `REM` (with parsed key-value pairs for DATE, GENRE, REPLAYGAIN_ALBUM_GAIN, etc.).
- Encoding: UTF-8 works; UTF-8 BOM — not documented (likely fails); Windows-1252 — passes raw bytes (issue #11: "Suppress bad character '�'?" still open).
- **Critical issue**: Open UAF (use-after-free) security bug in `CDText`, `REM`, and `Track` types (issue #19, filed Apr 2026). Missing lifetime specifiers.
- v3.0.1 fails to build on docs.rs; stick to v2.0.0.
- **Dealbreaker**: GPL-2.0 license (Reverie is MIT). Requires native `libcue` system library for build — complicates Windows cross-compilation (issue #10).

```rust
use cue::cd::CD;
let cd = CD::parse(cue_string)?;
for track in cd.tracks() {
    println!("{} - {} ({})", track.get_filename(), track.get_title(), track.get_start());
}
```

#### `rcue` (pure Rust, zero dependencies)
- **Recommended for Reverie**. MIT license, zero dependencies, tested with Japanese UTF-8 (マジコカタストロフィ) and standard ASCII.
- Parses: `FILE`, `TRACK`, `TITLE`, `PERFORMER`, `SONGWRITER`, `INDEX`, `PREGAP`, `REM`, `FLAGS`, `CATALOG`, `ISRC`.
- Encoding: UTF-8 confirmed working. UTF-8 BOM — not explicitly documented (likely fails without preprocessing). Windows-1252 — not tested.
- Known issues: indentation-insensitive parsing (REM fields at wrong nesting get misassigned), extraneous whitespace between fields can cause parse failures, escaped quotes are transformed.
- API: `parse_from_file(path, strict_mode)` → structured `Cue` object with tracks, files, comments.

```rust
use rcue::parser::parse_from_file;
let cue = parse_from_file("album.cue", true)?;
for track in cue.tracks {
    println!("{}: {:?}", track.track_num, track.title);
}
```

#### `cuna` (nom-based pure Rust)
- MIT license. Explicit UTF-8 BOM support (trims BOM before parsing).
- Parses: `TITLE`, `PERFORMER`, `SONGWRITER`, `FILE`, `TRACK`, `INDEX`, `FLAGS`, `ISRC`, `REM`, `PREGAP`, `POSTGAP`, `CATALOG`, `CDTEXTFILE`.
- Encoding: UTF-8 + UTF-8 BOM ✅. Windows-1252 — not tested/no explicit support.
- API described as "a bit complex"; breaking changes across minor versions (v0.6 had incompatible changes).

```rust
use cuna::Cuna;
let cue = Cuna::open("file.cue")?;
println!("Genre: {:?}", cue.comments.iter().find(|c| c.starts_with("GENRE")));
```

#### `cue_lib` (newest pure Rust, Codeberg)
- **Very promising but too new for production**. Published Apr 17, 2026 — 12 total downloads.
- MIT license, `no_std`-capable, feature-gated (`alloc`, `metadata`, `serde`, `ean`, `upc`).
- Based on original CDRWIN specification. 20 commits, single contributor.
- API: `CuesheetParser::new().parse(str)` → structured `Cuesheet` with tracks.
- Monitor for maturity over next 3–6 months; potential replacement for `rcue`.

### CUE Command Coverage Matrix

| Command | `cue` (libcue) | `rcue` | `cuna` | `cue_lib` | `cue_sheet` (archived) |
|---------|:---:|:---:|:---:|:---:|:---:|
| `TITLE` | ✅ | ✅ | ✅ | ✅ | ✅ |
| `PERFORMER` | ✅ | ✅ | ✅ | ✅ | ✅ |
| `SONGWRITER` | ✅ | ✅ | ✅ | ❓ | ✅ |
| `FILE` | ✅ | ✅ | ✅ | ✅ | ✅ |
| `TRACK` | ✅ | ✅ | ✅ | ✅ | ✅ |
| `INDEX` | ✅ | ✅ | ✅ | ✅ | ✅ |
| `PREGAP` | ✅ | ✅ | ✅ | ❓ | ✅ |
| `POSTGAP` | ✅ | ❌ | ✅ | ❓ | ✅ |
| `ISRC` | ✅ | ✅ | ✅ | ❓ | ✅ |
| `FLAGS` | ✅ | ✅ | ✅ | ❓ | ✅ |
| `CATALOG` | ✅ | ✅ | ✅ | ✅ | ✅ |
| `CDTEXTFILE` | ✅ | ❌ | ✅ | ❓ | ✅ |
| `REM` (generic) | ✅ | ✅ | ✅ | ✅ | ✅ |
| `REM DATE` | ✅ | ✅ | ✅ | ❓ | ❌ (flat) |
| `REM GENRE` | ✅ | ✅ | ✅ | ❓ | ❌ (flat) |
| `REM REPLAYGAIN_*` | ✅ | ❌ | ❓ | ❓ | ❌ (flat) |
| FILE types: WAVE | ✅ | ✅ | ✅ | ✅ | ✅ |
| FILE types: FLAC | ✅ (as WAVE) | ✅ (as WAVE) | ✅ (as WAVE) | ❓ | ✅ (as WAVE) |
| FILE types: APE | ✅ (as WAVE) | ✅ (as WAVE) | ✅ (as WAVE) | ❓ | ✅ (as WAVE) |
| FILE types: MP3 | ✅ | ✅ | ✅ | ❓ | ✅ |

### Encoding Handling

| Encoding | `cue` (libcue) | `rcue` | `cuna` |
|----------|:---:|:---:|:---:|
| UTF-8 | ✅ | ✅ (tested) | ✅ |
| UTF-8 BOM | ❌ (unlikely) | ❌ (likely fails) | ✅ (explicit strip) |
| Windows-1252 | ⚠️ (raw bytes, replacement chars) | ❌ (not tested) | ❌ (not tested) |
| ISO-8859-1 | ⚠️ (raw bytes pass through) | ❌ (not tested) | ❌ (not tested) |

### Recommendation

| Priority | Crate | Rationale |
|----------|-------|-----------|
| **Primary** | `rcue` v0.1.3 | MIT license, zero deps, structured API, proven UTF-8, adequate command coverage |
| **Fallback** | `cuna` v0.7.0 | MIT, explicit BOM support, slightly better encoding handling |
| **Monitor** | `cue_lib` | Active development, MIT, no_std; too new for production today |
| **Avoid** | `cue` / `cue-sys` | GPL-2.0 incompatible with MIT; requires native C library; has open UAF security bug |

---

## lofty FLAC CUESHEET Support

### ❌ lofty 0.21 does NOT support FLAC CUESHEET metadata blocks

**Evidence:**

1. **TagType enum lacks CueSheet**: Only `Ape`, `Id3v1`, `Id3v2`, `Mp4Ilst`, `VorbisComments`, `RiffInfo`, `AiffText` exist.

2. **FlacFile struct has no CUESHEET field**:
   ```rust
   pub struct FlacFile {
       pub(crate) id3v2_tag: Option<Id3v2Tag>,
       pub(crate) vorbis_comments_tag: Option<VorbisComments>,
       pub(crate) pictures: Vec<(Picture, PictureInformation)>,
       pub(crate) properties: FlacProperties,
   }
   // No cuesheet field
   ```

3. **FLAC block parser skips block type 5** (CUESHEET):
   ```rust
   // Only these block IDs are defined:
   pub(in crate::flac) const BLOCK_ID_STREAMINFO: u8 = 0;
   pub(in crate::flac) const BLOCK_ID_PADDING: u8 = 1;
   pub(in crate::flac) const BLOCK_ID_SEEKTABLE: u8 = 3;
   pub(in crate::flac) const BLOCK_ID_VORBIS_COMMENTS: u8 = 4;
   pub(in crate::flac) const BLOCK_ID_PICTURE: u8 = 6;
   // BLOCK_ID_CUESHEET = 5 is NOT defined — skipped during parsing
   ```

4. **GitHub Discussion #285** confirms this is a known gap (Nov 2023). Maintainer acknowledged it's "simple enough to parse" but hasn't implemented it due to architectural constraints around tag-centric design. Status: **not implemented**.

### Alternative: `flac` crate (sourrust) v0.5.0

The `flac` crate natively reads FLAC CUESHEET metadata blocks:

```rust
// Cargo.toml
// flac = "0.5.0"

use flac::metadata::get_cue_sheet;

// Read embedded CUESHEET from a FLAC file
let cue_sheet = get_cue_sheet("track.flac")?;
if let Some(cue) = cue_sheet {
    println!("Media catalog: {}", cue.media_catalog_number);
    println!("Lead-in samples: {}", cue.lead_in_samples);
    for track in &cue.tracks {
        println!("Track {} offset={}", track.track_offset, track.track_number);
        for idx in &track.indices {
            println!("  INDEX {:02} offset={}", idx.index_point, idx.index_offset);
        }
    }
}
```

**Approach**: Use **`flac`** crate for reading CUESHEET metadata blocks and **`lofty`** for tag reading in the same project. They target different FLAC metadata structures and do not conflict.

### Alternative: Raw FLAC parsing

If adding `flac` as a dependency is undesirable, the FLAC CUESHEET block structure is well-defined (FLAC specification, METADATA_BLOCK_CUESHEET) and could be parsed manually with ~100 lines of raw byte reading code combined with OpenDAL's range read. This is not recommended — the `flac` crate handles edge cases and CRC validation.

### Other FLAC CUESHEET tools

- **`flac-tracksplit`**: Uses embedded CUESHEET to split FLAC files into individual tracks. Built on `metaflac`. Not relevant for metadata extraction.
- **`metaflac`**: Lower-level FLAC metadata manipulation. Overkill for read-only CUESHEET extraction.

---

## OpenDAL Range Read Support

### ✅ OpenDAL 0.55 fully supports byte-range reads

Both `services-fs` and `services-memory` backends support range reads. No additional dependencies or features needed.

### API

```rust
use opendal::{Operator, services::Fs};

let op = Operator::new(Fs::default().root("/music"))?.finish();

// Read bytes 100-200 (exclusive range end)
let data = op.read_with("large_file.flac")
    .range(100..201)
    .await?;

// Also supported via reader():
let reader = op.reader("large_file.flac").await?;
let data = reader.read(100..201).await?;
```

### Backend Support

| Backend | Range Read | Evidence |
|---------|:---:|----------|
| `services-fs` | ✅ | `FsBackend::read()` passes `args.range()` to file operations |
| `services-memory` | ✅ | `MemoryBackend::read()` slices content: `args.range().to_range_as_usize()` |
| Other backends | ✅ | Range reads are a core `OpRead` feature, supported by all read-capable backends |

### Usage in Reverie Context

For reading CUESHEET data from FLAC files stored via OpenDAL:
1. The FLAC CUESHEET metadata block follows the STREAMINFO block (always first 38 bytes of the FLAC file, after "fLaC" magic).
2. Use `op.read_with(path).range(start, end).await?` to read the CUESHEET block when detected via block header scanning.
3. This avoids loading an entire multi-gigabyte FLAC file just to extract a ~1KB CUESHEET block.

---

## Recommended Approach

### CUE Sheet Parsing (.cue files)

```
Primary:   rcue v0.1.3  (MIT, zero-deps, structured API, proven UTF-8)
Fallback:  cuna v0.7.0  (MIT, explicit UTF-8 BOM support)
Monitor:   cue_lib      (active development; re-evaluate in 3-6 months)
Avoid:     cue / cue-sys (GPL-2.0 license conflict, native library dependency)
```

**Encoding mitigation**: Preprocess file bytes to strip BOM (0xEF 0xBB 0xBF) and try parsing as UTF-8. If parsing fails, fall back to Windows-1252 → UTF-8 conversion via `encoding_rs` before retrying. This is a common pattern in CUE processing and neither `rcue` nor `cuna` handle it natively.

### FLAC Embedded CUESHEET

```
Primary:   flac crate v0.5.0  (alongside lofty for VorbisComments/ID3v2)
```

### Byte-Range Reads

```
Primary:   opendal v0.55  op.read_with(path).range(start..end).await?
          (already in deps; services-fs and services-memory both supported)
```

### Dependency Additions (tentative)

```toml
# CUE file parsing (.cue files)
rcue = "0.1.3"

# FLAC embedded CUESHEET metadata
flac = "0.5"

# Encoding conversion (for non-UTF-8 CUE files)
encoding_rs = "0.8"
```

No changes needed for OpenDAL — range reads work with existing v0.55 setup.

---

## Appendix: Research Data

### Agent Research Summary

| Agent | Task | Result |
|-------|------|--------|
| librarian (bg_64357485) | `cue` crate research | 6 crates evaluated; comprehensive per-field detail on `cue` (libcue bindings) |
| librarian (bg_176d5db0) | `cue-sheet` + other crates | Found 6 crates total; `cue_lib` discovered as newest (Apr 2026); `cue_sheet` confirmed archived |
| librarian (bg_6f93a970) | lofty FLAC CUESHEET | Confirmed NO support with source evidence; `flac` crate identified as alternative |
| librarian (bg_738892a1) | OpenDAL range read | Confirmed YES support with source evidence; both services-fs and services-memory verified |

### Source References

- `cue` crate: [crates.io](https://crates.io/crates/cue) | [GitHub](https://github.com/mistydemeo/libcue.rs)
- `rcue` crate: [crates.io](https://crates.io/crates/rcue) | [GitHub](https://github.com/gyng/rcue)
- `cuna` crate: [crates.io](https://crates.io/crates/cuna) | [GitHub](https://github.com/snylonue/cuna)
- `cue_sheet` crate: [crates.io](https://crates.io/crates/cue_sheet) | [GitHub](https://github.com/leoschwarz/cue_sheet) (ARCHIVED)
- `cue_lib` crate: [Codeberg](https://codeberg.org/SuperiorOne/cue_lib)
- `flac` crate: [crates.io](https://crates.io/crates/flac) | [GitHub](https://github.com/sourrust/flac)
- lofty CUESHEET discussion: [GitHub #285](https://github.com/Serial-ATA/lofty-rs/discussions/285)
- OpenDAL range read: [docs.rs](https://docs.rs/opendal/0.55.0/opendal/struct.Operator.html#method.read_with)
- CUE format reference: [Wikipedia](https://en.wikipedia.org/wiki/Cue_sheet_(computing))
- FLAC CUESHEET spec: [FLAC format documentation](https://xiph.org/flac/format.html#metadata_block_cuesheet)
