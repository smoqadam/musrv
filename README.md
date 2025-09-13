musrv — minimal music server

Overview
- Scans a directory for audio files (mp3, flac, m4a, ogg, opus, etc.)
- Serves a simple web UI at `/` with albums and a basic player
- Exposes playlists:
  - `/library.m3u8` — the entire library
  - `/album/<AlbumName>.m3u8` — a single album
- JSON API: `/library.json` — albums + tracks for the UI
- Streams files with HTTP Range support (seek works in most players)
- Safe-by-default static serving (no traversal; hides dotfiles)
- Hot refresh: `GET /admin/rescan` rescans and atomically swaps the library

Install

Cargo (from source)
- Requirements: Rust stable
- Clone and build:
  - `git clone https://github.com/smoqadam/musrv`
  - `cd musrv`
  - `cargo install --path .`  (installs the `musrv` binary to Cargo bin dir)

Build from source
- `git clone https://github.com/smoqadam/musrv`
- `cd musrv`
- `cargo build --release`
- Binary at `target/release/musrv`

Docker
- Pull image:
  - `docker pull ghcr.io/smoqadam/musrv:latest`
- Run, serving `/music` on port 8080:
  - `docker run --rm -p 8080:8080 -v /path/to/music:/music ghcr.io/smoqadam/musrv:latest serve /music --bind 0.0.0.0 --port 8080`

Usage
- Serve a directory:
  - `musrv serve /path/to/music --bind 0.0.0.0 --port 8080`
- Nested albums (full parent path):
  - `musrv serve /path/to/music --album-depth 0`
- Two-level album keys (e.g., Artist/Album):
  - `musrv serve /path/to/music --album-depth 2`
- Open the UI:
  - `http://<LAN-IP>:8080/`
- Playlists:
  - Library: `http://<LAN-IP>:8080/library.m3u8`
  - Album: `http://<LAN-IP>:8080/album/<AlbumName>.m3u8`
- Rescan (optional):
  - `GET http://<LAN-IP>:8080/admin/rescan` (the UI has a Rescan button)

Notes
- When binding to `0.0.0.0`, musrv prints a LAN-accessible URL for convenience.
- Album grouping defaults to first-level directories (`--album-depth 1`). Set `--album-depth 0` to group by full parent path or `--album-depth N` to use the first N path components.
- Files directly under the root are grouped into a virtual "Singles" album (or "Singles (root)" if that name collides).
- Hidden/system paths are filtered by default.

Development
- Lint and format: `cargo fmt --all` and `cargo clippy -- -D warnings`
- Tests: `cargo test --all`

CI
- GitHub Actions runs fmt, clippy, tests on every push/PR
- Docker image is built and pushed to `ghcr.io/smoqadam/musrv` on pushes to the default branch and version tags (`v*.*.*`)
