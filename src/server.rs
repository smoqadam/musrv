use std::path::PathBuf;
use std::sync::Arc;

use axum::{extract::{Path as AxPath, State}, http::{header, StatusCode}, response::IntoResponse, routing::get, Router};
use tower_http::{services::ServeDir, trace::TraceLayer};

use crate::library::{Album, Library, Track};
use crate::playlist::{encode_path, render_m3u8};
use minijinja::{context, Environment};
use serde::Serialize;

#[derive(Clone)]
pub struct AppState {
    pub lib: Arc<Library>,
    pub base: String,
    pub root: PathBuf,
}

pub fn build_router(state: AppState) -> Router {
    let files = ServeDir::new(state.root.clone());
    Router::new()
        .route("/", get(index))
        .route("/library.m3u8", get(library_m3u8))
        .route("/album/*name", get(album_m3u8))
        .fallback_service(files)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

#[derive(Serialize)]
struct TplTrack { title: String, url: String }

#[derive(Serialize)]
struct TplAlbum { name: String, tracks: Vec<TplTrack> }

async fn index(State(state): State<AppState>) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut env = Environment::new();
    if let Err(e) = env.add_template("index.html", include_str!("static/index.html")) {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("template error: {}", e)));
    }
    let tmpl = match env.get_template("index.html") {
        Ok(t) => t,
        Err(e) => return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("template error: {}", e))),
    };
    let mut albums_tpl = Vec::new();
    for a in state.lib.albums() {
        let mut ts = Vec::new();
        for t in &a.tracks {
            let rel = t.path.to_string_lossy().replace('\\', "/");
            let url = format!("{}{}", state.base, encode_path(&rel));
            let title = t.path.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string();
            ts.push(TplTrack { title, url });
        }
        albums_tpl.push(TplAlbum { name: a.name.clone(), tracks: ts });
    }
    let body = match tmpl.render(context!(albums => albums_tpl)) {
        Ok(s) => s,
        Err(e) => return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("render error: {}", e))),
    };
    Ok(([
        (header::CONTENT_TYPE, "text/html; charset=utf-8"),
        (header::CACHE_CONTROL, "no-cache"),
    ], body))
}

async fn library_m3u8(State(state): State<AppState>) -> impl axum::response::IntoResponse {
    let body = render_m3u8(&state.base, &state.root, state.lib.tracks());
    ([
        (header::CONTENT_TYPE, "audio/x-mpegurl; charset=utf-8"),
        (header::CACHE_CONTROL, "no-cache"),
    ], body)
}

async fn album_m3u8(AxPath(mut name): AxPath<String>, State(state): State<AppState>) -> impl axum::response::IntoResponse {
    if let Some(stripped) = name.strip_suffix(".m3u8") { name = stripped.to_string(); }
    if let Ok(decoded) = urlencoding::decode(&name) { name = decoded.into_owned(); }
    if let Some(album) = state.lib.album_by_name(&name) {
        let body = render_m3u8(&state.base, &state.root, &album.tracks);
        ([
            (header::CONTENT_TYPE, "audio/x-mpegurl; charset=utf-8"),
            (header::CACHE_CONTROL, "no-cache"),
        ], body)
    } else {
        ([
            (header::CONTENT_TYPE, "text/plain; charset=utf-8"),
            (header::CACHE_CONTROL, "no-cache"),
        ], String::from("#EXTM3U\n"))
    }
}
