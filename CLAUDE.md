# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Oxidant is a high-performance audio metadata library written in Rust with Python bindings via PyO3. It supports reading and modifying metadata for ID3 (v1/v2) and FLAC audio formats by manually parsing audio file byte streams according to official specifications.

## Build and Development Commands

```bash
# Build and install the library (must use uv)
uv run maturin develop

# Run tests
uv run python test/test_simple.py
uv run python test/test_*.py  # Run all tests

# Check Python environment
uv python pin 3.12.9  # Set Python version if needed
```

**Important**: Always use `uv run maturin develop` to build. The project uses uv for Python environment management and maturin for building the Rust-Python extension.

## Architecture

### Core Modules

The codebase is organized around audio format parsers, with a unified Python API:

**`src/lib.rs`**: PyO3 binding entry point exposing `AudioFile`, `Metadata`, and `CoverArt` classes to Python.

**`src/id3/`**: ID3 tag parsing and manipulation
- `mod.rs` - Module exports
- `v1.rs` - ID3v1.0/v1.1 implementation (fixed 128-byte tag at end of file)
- `v2.rs` - ID3v2 tag implementation with header/frame structure
- `frames.rs` - Frame-specific decoding (APIC for pictures, USLT for unsynchronized lyrics, TIT2/TPE1/etc for text)

**`src/flac/`**: FLAC metadata handling
- `mod.rs` - Module exports
- `metadata.rs` - METADATA_BLOCK parsing (chained structure before audio data)
- `vorbis.rs` - VORBIS_COMMENT block parsing
- `picture.rs` - PICTURE block parsing

**`src/utils/`**: Shared utilities
- `encoding.rs` - Text encoding conversions (UTF-8, UTF-16, UTF-16BE, ISO-8859-1)
- `io.rs` - Byte stream I/O utilities

### Format-Specific Details

**ID3v2**: Frame headers differ by version:
- v2.2: 3-byte frame IDs
- v2.3: Regular big-endian 32-bit frame sizes
- v2.4: Synchsafe integers (7 bits per byte) for frame sizes

**FLAC**: Uses big-endian encoding. Metadata blocks are chained with:
- Block header (1 byte: last flag + block type, 3 bytes: size)
- Block data
- Last metadata block has MSB set, followed by audio data

### Python API

```python
# Detect and read
audio = AudioFile("path/to/file.mp3")
metadata = audio.read_metadata()
cover = audio.extract_cover()

# Modify metadata
audio.set_cover("image.jpg", "image/jpeg", "Cover")
audio.set_lyrics("Lyrics text...")
audio.remove_lyrics()
```

### Key Implementation Patterns

1. **Manual byte parsing**: All formats are parsed by reading byte streams directly, following specifications exactly. No external metadata libraries are used.

2. **File modification**: For tag writing/modification, the entire file is read into memory, modified, and rewritten. This is simpler but less memory-efficient for large files.

3. **Encoding handling**: ID3 text frames start with a byte indicating encoding (0=ISO-8859-1, 1=UTF-16, 2=UTF-16BE, 3=UTF-8). The `utils::encoding` module handles conversions.

4. **Synchsafe integers**: ID3v2 uses 7-bit per byte integers for tag size calculations. The `to_synchsafe()` and `parse_synchsafe()` functions handle this.

## Testing

Test files are in the `test/` directory. Tests use real audio files and exercise the full Python API. Key test areas:
- Simple metadata reading
- Cover art extraction and setting
- Lyrics reading, setting, and removal
- Synchsafe integer conversion

## Current Implementation Status

- **ID3v1/v2**: Read and write metadata, cover art, lyrics (USLT frame)
- **FLAC**: Read and write metadata, cover art, lyrics
- Known issue: ID3v2 tag cover art handling may have issues (per recent commit message)