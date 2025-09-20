# musrv

Minimal, zero-config music server

## Get started

```sh

# docker
$ docker run --rm -p 8080:8080 -d -v /my/music/library:/music --pull always \
     ghcr.io/smoqadam/musrv:latest-armv7 serve /music --bind 0.0.0.0 \
     --public-url http://localhost:8030/
```

Or: 

```sh
# install
$ curl -fsSL https://raw.githubusercontent.com/smoqadam/musrv/main/install.sh | sh

# run
$ musrv serve /Users/saeed/Downloads --bind 0.0.0.0 --port 8080 --qr
root: /Users/saeed/Downloads
listen: http://192.168.178.33:8080
tracks: 101
ui: http://192.168.178.33:8080

scan to open UI:


    █▀▀▀▀▀█ █▀▀██▄▄ █ █▀▀▀▀▀█
    █ ███ █ ▀██  █▄   █ ███ █
    █ ▀▀▀ █ ▀█▀▄▄██   █ ▀▀▀ █
    ▀▀▀▀▀▀▀ ▀ ▀ █▄█ ▀ ▀▀▀▀▀▀▀
    █  ▀▀▀▀█▀▄███  █▄█  █▄██▀
     █▀▀ █▀█████ ▀██ █▀▀██▄▄█
    ██▀▄█▄▀▄ █ ▀█▄▀▄▀▀▀▄▄  ▄▀
    █▀▄ ▄▀▀██▄█ █  ▀█▀  ▄▀█▀█
    ▀ ▀▀  ▀▀▄█▀█▄██▄█▀▀▀█ ▀█
    █▀▀▀▀▀█ █ ▄ █▄█ █ ▀ █▀  ▀
    █ ███ █ █  ██▀ ▀▀▀██▀█ ██
    █ ▀▀▀ █  ▄▀▀▄ ▀▄ ▄▄▄█ ███
    ▀▀▀▀▀▀▀ ▀  ▀ ▀▀▀▀ ▀  ▀  ▀


```

Open [http://localhost:8080/](http://localhost:8080/) (or the printed LAN URL) in your browser.

---

## What you get

<img width="300" height="1490" alt="image" src="https://github.com/user-attachments/assets/7a75a267-a7d9-43dd-bbb7-37a6eca17ba0" />


* Lightweight web UI with folders and a simple player
* Auto-scan of folders for audio (`mp3`, `flac`, `m4a`, `ogg`, `opus`, …)
* Generates M3U8 playlists you can feed into players such as **Apple Music, VLC, foobar2000**, and others:
     - Per-folder: `http://localhost:8080/api/folder.m3u8?path=<Folder/Path>`

---

## What you don’t get

musrv is deliberately minimal. It **does not**:

* Fetch metadata from the internet
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
  
  docker run --rm -p 8080:8080 -d -v /my/music/library:/music \
       ghcr.io/smoqadam/musrv:latest-amd64 serve /music --bind 0.0.0.0 \
       --public-url http://localhost:8080/


  # arm
  docker run --rm -p 8080:8080 -d -v /my/music/library:/music \
       ghcr.io/smoqadam/musrv:latest-armv7 serve /music --bind 0.0.0.0 \
       --public-url http://localhost:8080/

  ```

Then open http://localhost:8080.

---

## Advanced usage

* Rescan: `GET /admin/rescan`

---

## Notes

* Hidden/system files are ignored; symlinks are not followed.
* Binding to `0.0.0.0` exposes a LAN URL that’s also used in playlists. When running behind Docker or a reverse proxy, pass `--public-url http://your-host:port/` to control the advertised URLs.

---

## Known Limitations

- VLC on iPhone
  The playlist view does not show individual songs, but playback still works sequentially (tracks play one after another).

- iPhone lock screen controls
  The Next/Previous buttons on the lock screen do not function. As far as I know, this is due to iOS limitations.
If you know a workaround, feel free to open an issue

## Todo
- [ ] Add basic authentication
---

## Development

```sh
cargo fmt --all
cargo clippy -- -D warnings
cargo test --all
```
