# Real Audio Test Fixtures

This directory contains real audio files for testing the Reverie music server scanner.

## Files

| File | Source | Artist | Duration | Size |
|------|--------|--------|----------|------|
| `classical_sample.mp3` | FreePD.com (via CC0-1.0-Music) | Unknown | ~3:30 | 4.6 MB |
| `sample.mp3` | FreePD.com (via CC0-1.0-Music) | Unknown | ~2:30 | 3.5 MB |

## License

All files in this directory are released under **CC0 1.0 Universal (Public Domain)** license.

- Source Repository: [SoundSafari/CC0-1.0-Music](https://github.com/SoundSafari/CC0-1.0-Music)
- Original Source: [FreePD.com](https://freepd.com/) (now offline)
- License: [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/)

You can use, copy, modify, distribute, and perform these files without permission or attribution.

## Track Information

### classical_sample.mp3
- **Title**: Brandenburg Concerto III, Allegro I
- **Genre**: Classical
- **Source URL**: https://github.com/SoundSafari/CC0-1.0-Music/tree/main/freepd.com
- **Downloaded**: 2026-04-29

### sample.mp3
- **Title**: Adventure
- **Genre**: Cinematic/Adventure
- **Source URL**: https://github.com/SoundSafari/CC0-1.0-Music/tree/main/freepd.com
- **Downloaded**: 2026-04-29

## How to Re-download

If these files need to be re-downloaded:

```bash
# Clone the CC0-1.0-Music repository
git clone https://github.com/SoundSafari/CC0-1.0-Music.git

# Copy desired files
cp CC0-1.0-Music/freepd.com/Brandenburg\ Concerto\ III,\ Allegro\ I.mp3 tests/fixtures/audio/real/classical_sample.mp3
cp CC0-1.0-Music/freepd.com/Adventure.mp3 tests/fixtures/audio/real/sample.mp3
```

## Notes

- These files are used for scanner integration tests
- Both files are parseable by the `lofty` audio metadata library
- Files are intentionally kept under 15MB to avoid Git LFS issues