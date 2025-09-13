musrv
Minimal, zero‑config music server.

## Overview
- Scans a folder for audio files (mp3, flac, m4a, ogg, opus, …)
- Serves a small web UI at `/` with albums and a basic player
- Generates playlists and streams files with HTTP Range (seek works)

## Key Feature
- Playlists: `/library.m3u8`, `/album/<AlbumName>.m3u8`
- JSON: `/library.json` (albums + tracks for the UI)
- Static streaming under the chosen root with traversal protection
- Rescan endpoint: `GET /admin/rescan` (atomic swap of the library)
- Album grouping by path depth; root files collected as “Singles”
  
<img width="300" height="1490" alt="Image" src="https://github.com/user-attachments/assets/2f384f12-61f8-4c6c-9b00-128092ade823" />


## Install
- Cargo (from source):
  - `git clone https://github.com/smoqadam/musrv && cd musrv`
  - `cargo install --path .`
- Docker:
  - `docker pull ghcr.io/smoqadam/musrv:latest`
- Install script (latest release):
  - `curl -fsSL https://raw.githubusercontent.com/smoqadam/musrv/main/install.sh | sh`
  - Set a specific version with `MUSRV_VERSION=v0.1.0`.

## Quick Start
- Binary: `musrv serve /path/to/music --bind 0.0.0.0 --port 8080`
- Docker: `docker run --rm -p 8080:8080 -v /music:/music ghcr.io/smoqadam/musrv:latest serve /music --bind 0.0.0.0 --port 8080`
- Open: `http://<LAN-IP>:8080/`

## Usage
- Album depth (default 1):
  - Full parent path: `--album-depth 0`
  - First N components: `--album-depth N`
- Endpoints:
  - UI: `/`
  - JSON: `/library.json`
  - Playlists: `/library.m3u8`, `/album/<AlbumName>.m3u8`
  - Rescan: `GET /admin/rescan`
- External players:
  - Feed `http://<LAN-IP>:8080/library.m3u8` (or any album M3U8) to players that support online M3U/M3U8 like Apple Music, VLC, foobar2000, etc.

## Notes
- Binding to `0.0.0.0` prints a LAN URL; playlists also use a LAN IP.
- Root-level files appear in a virtual “Singles” album (or “Singles (root)” when colliding).
- Hidden/system files are ignored; symlinks are not followed.

## Development

- Format/lint/tests: `cargo fmt --all`, `cargo clippy -- -D warnings`, `cargo test --all`
- CI: PRs run fmt + clippy + tests. Pushes build multi‑arch Docker images to `ghcr.io/smoqadam/musrv`.
