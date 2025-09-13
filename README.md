# musrv

Minimal, zero-config music server

## Get started

```sh
curl -fsSL https://raw.githubusercontent.com/smoqadam/musrv/main/install.sh | sh
musrv serve /path/to/music
```

Open [http://localhost:8080/](http://localhost:8080/) in your browser.

---

## What you get

<img width="300" height="1490" alt="Screenshot" src="https://github.com/user-attachments/assets/2f384f12-61f8-4c6c-9b00-128092ade823" />

* Lightweight web UI with albums and a simple player
* Auto-scan of folders for audio (`mp3`, `flac`, `m4a`, `ogg`, `opus`, …)
* Generates M3U8 playlists you can feed into players such as **Apple Music, VLC, foobar2000**, and others:
     - Whole library: `http://localhost<LAN-IP>:8080/library.m3u8`
     - Per-album: `http://localhost:8080/album/<FolderName>.m3u8`

Albums are grouped by folder; root-level files are collected under **Singles**.

---

## What you don’t get

musrv is deliberately minimal. It **does not**:

* Fetch metadata from the internet
* Show album covers or artist info
* Download, sync, or manage your files

It simply turns your local music directory into a streaming server—fast, clean, and nothing more.

---

## Other ways to install

* From source:

  ```sh
  git clone https://github.com/smoqadam/musrv && cd musrv
  cargo install --path .
  ```
* Docker:

  ```sh
  docker run --rm -p 8080:8080 -v /music:/music \
    ghcr.io/smoqadam/musrv:latest serve /music
  ```

---

## Advanced usage

* Album depth (default 1):

  * Full parent path: `--album-depth 0`
  * First N components: `--album-depth N`
* JSON endpoint: `/library.json`
* Rescan: `GET /admin/rescan`

---

## Notes

* Hidden/system files are ignored; symlinks are not followed.
* Binding to `0.0.0.0` exposes a LAN URL that’s also used in playlists.

---

## Todo
- [ ] Add basic authentication
---

## Development

```sh
cargo fmt --all
cargo clippy -- -D warnings
cargo test --all
```
