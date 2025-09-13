use std::path::PathBuf;
use std::sync::Arc;

use axum::{
    Router,
    extract::{Path as AxPath, State},
    http::{StatusCode, header, HeaderValue},
    response::IntoResponse,
    routing::get,
};
use tokio_util::io::ReaderStream;
use tower_http::trace::TraceLayer;

use crate::library::Library;
use crate::playlist::{encode_path, render_m3u8};
use minijinja::{Environment, context};
use serde::Serialize;

#[derive(Clone)]
pub struct AppState {
    pub lib: Arc<Library>,
    pub base: String,
    pub root: PathBuf,
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/library.m3u8", get(library_m3u8))
        .route("/album/*name", get(album_m3u8))
        .route("/*path", get(static_file))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

#[derive(Serialize)]
struct TplTrack {
    title: String,
    url: String,
}

#[derive(Serialize)]
struct TplAlbum {
    name: String,
    tracks: Vec<TplTrack>,
}

async fn index(State(state): State<AppState>) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut env = Environment::new();
    if let Err(e) = env.add_template("index.html", include_str!("static/index.html")) {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("template error: {e}"),
        ));
    }
    let tmpl = match env.get_template("index.html") {
        Ok(t) => t,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("template error: {e}"),
            ));
        }
    };
    let mut albums_tpl = Vec::new();
    for a in state.lib.albums() {
        let mut ts = Vec::new();
        for t in &a.tracks {
            let rel = t.path.to_string_lossy().replace('\\', "/");
            let url = format!("{}{}", state.base, encode_path(&rel));
            let title = t
                .path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            ts.push(TplTrack { title, url });
        }
        albums_tpl.push(TplAlbum {
            name: a.name.clone(),
            tracks: ts,
        });
    }
    let body = match tmpl.render(context!(albums => albums_tpl)) {
        Ok(s) => s,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("render error: {e}"),
            ));
        }
    };
    Ok((
        [
            (header::CONTENT_TYPE, "text/html; charset=utf-8"),
            (header::CACHE_CONTROL, "no-cache"),
        ],
        body,
    ))
}

async fn library_m3u8(State(state): State<AppState>) -> impl axum::response::IntoResponse {
    let body = render_m3u8(&state.base, &state.root, state.lib.tracks());
    (
        [
            (header::CONTENT_TYPE, "audio/x-mpegurl; charset=utf-8"),
            (header::CACHE_CONTROL, "no-cache"),
        ],
        body,
    )
}

async fn album_m3u8(
    AxPath(mut name): AxPath<String>,
    State(state): State<AppState>,
) -> impl axum::response::IntoResponse {
    if let Some(stripped) = name.strip_suffix(".m3u8") {
        name = stripped.to_string();
    }
    if let Ok(decoded) = urlencoding::decode(&name) {
        name = decoded.into_owned();
    }
    if decoded_album_invalid(&name) {
        return (
            [
                (header::CONTENT_TYPE, "text/plain; charset=utf-8"),
                (header::CACHE_CONTROL, "no-cache"),
            ],
            String::from("#EXTM3U\n"),
        );
    }
    if let Some(album) = state.lib.album_by_name(&name) {
        let body = render_m3u8(&state.base, &state.root, &album.tracks);
        (
            [
                (header::CONTENT_TYPE, "audio/x-mpegurl; charset=utf-8"),
                (header::CACHE_CONTROL, "no-cache"),
            ],
            body,
        )
    } else {
        (
            [
                (header::CONTENT_TYPE, "text/plain; charset=utf-8"),
                (header::CACHE_CONTROL, "no-cache"),
            ],
            String::from("#EXTM3U\n"),
        )
    }
}

fn decoded_album_invalid(name: &str) -> bool {
    name.is_empty() || name.starts_with('.') || name.contains('/') || name.contains('\\')
}

async fn static_file(
    AxPath(path): AxPath<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if path.is_empty() || path == "/" {
        return Err((StatusCode::NOT_FOUND, String::new()));
    }
    let decoded = match urlencoding::decode(&path) {
        Ok(s) => s.into_owned(),
        Err(_) => return Err((StatusCode::BAD_REQUEST, String::from("bad encoding"))),
    };
    if decoded.contains("..") || decoded.starts_with('/') || decoded.contains('\0') {
        return Err((StatusCode::NOT_FOUND, String::new()));
    }
    for seg in decoded.split('/') {
        if seg.is_empty() {
            continue;
        }
        if seg.starts_with('.')
            || seg == "Thumbs.db"
            || seg == "desktop.ini"
            || seg.starts_with("._")
        {
            return Err((StatusCode::NOT_FOUND, String::new()));
        }
    }
    let abs = state.root.join(&decoded);
    let abs = match tokio::fs::canonicalize(&abs).await {
        Ok(p) => p,
        Err(_) => return Err((StatusCode::NOT_FOUND, String::new())),
    };
    let root = &state.root;
    if !abs.starts_with(root) {
        return Err((StatusCode::NOT_FOUND, String::new()));
    }
    let file = match tokio::fs::File::open(&abs).await {
        Ok(f) => f,
        Err(_) => return Err((StatusCode::NOT_FOUND, String::new())),
    };
    let stream = ReaderStream::new(file);
    let body = axum::body::Body::from_stream(stream);
    let mime = mime_guess::from_path(&abs).first_or_octet_stream();
    let ct = HeaderValue::from_str(mime.as_ref()).unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream"));
    Ok(([(header::CONTENT_TYPE, ct)], body))
}
