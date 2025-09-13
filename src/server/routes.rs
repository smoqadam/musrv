use axum::Json;
use axum::http::Request;
use axum::{
    Router,
    extract::{Path as AxPath, State},
    http::{StatusCode, header},
    response::IntoResponse,
    routing::get,
};
use serde::Serialize;
use std::sync::Arc;
use tower::util::ServiceExt;
use tower_http::{services::ServeFile, trace::TraceLayer};

use crate::playlist::render_m3u8;

use super::{helpers, state::AppState};

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/library.m3u8", get(library_m3u8))
        .route("/library.json", get(library_json))
        .route("/admin/rescan", get(admin_rescan))
        .route("/album/*name", get(album_playlist))
        .route("/*path", get(static_file))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn index(State(_state): State<AppState>) -> Result<impl IntoResponse, (StatusCode, String)> {
    let body = include_str!("../static/index.html");
    Ok((
        [
            (header::CONTENT_TYPE, "text/html; charset=utf-8"),
            (header::CACHE_CONTROL, "no-cache"),
        ],
        body.to_string(),
    ))
}

async fn library_m3u8(State(state): State<AppState>) -> impl axum::response::IntoResponse {
    let lib = state.lib.load();
    let body = render_m3u8(&state.base, &state.root, lib.tracks());
    (
        [
            (header::CONTENT_TYPE, "audio/x-mpegurl; charset=utf-8"),
            (header::CACHE_CONTROL, "no-cache"),
        ],
        body,
    )
}

#[derive(Serialize)]
struct JsonTrack {
    path: String,
    size: Option<u64>,
}
#[derive(Serialize)]
struct JsonAlbum {
    name: String,
    tracks: Vec<JsonTrack>,
}
#[derive(Serialize)]
struct JsonLibrary {
    albums: Vec<JsonAlbum>,
    tracks: Vec<JsonTrack>,
}

async fn library_json(State(state): State<AppState>) -> impl axum::response::IntoResponse {
    let lib = state.lib.load();
    let tracks: Vec<JsonTrack> = lib
        .tracks()
        .iter()
        .map(|t| JsonTrack {
            path: t.path.to_string_lossy().replace('\\', "/"),
            size: t.size,
        })
        .collect();
    let albums: Vec<JsonAlbum> = lib
        .albums()
        .iter()
        .map(|a| JsonAlbum {
            name: a.name.clone(),
            tracks: a
                .tracks
                .iter()
                .map(|t| JsonTrack {
                    path: t.path.to_string_lossy().replace('\\', "/"),
                    size: t.size,
                })
                .collect(),
        })
        .collect();
    let body = JsonLibrary { albums, tracks };
    Json(body)
}

async fn album_playlist(
    AxPath(name): AxPath<String>,
    State(state): State<AppState>,
) -> impl axum::response::IntoResponse {
    let Ok(album_name) = helpers::parse_album_name(name) else {
        return (
            [
                (header::CONTENT_TYPE, "text/plain; charset=utf-8"),
                (header::CACHE_CONTROL, "no-cache"),
            ],
            String::new(),
        );
    };
    let lib = state.lib.load();
    if let Some(album) = lib.album_by_name(&album_name) {
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
            String::new(),
        )
    }
}

async fn static_file(
    AxPath(path): AxPath<String>,
    State(state): State<AppState>,
    req: Request<axum::body::Body>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let decoded = helpers::validate_request_path(&path)
        .map_err(|_| (StatusCode::NOT_FOUND, String::new()))?;
    let abs = state.root.join(&decoded);
    let abs = match tokio::fs::canonicalize(&abs).await {
        Ok(p) => p,
        Err(_) => return Err((StatusCode::NOT_FOUND, String::new())),
    };
    if !abs.starts_with(&state.root) {
        return Err((StatusCode::NOT_FOUND, String::new()));
    }
    let svc = ServeFile::new(abs);
    let res = svc
        .oneshot(req)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, String::new()))?;
    Ok(res)
}

async fn admin_rescan(State(state): State<AppState>) -> impl axum::response::IntoResponse {
    let root = state.root.clone();
    let depth = state.album_depth;
    let new_lib = match tokio::task::spawn_blocking(move || {
        crate::library::Library::scan_with_depth(root, depth)
    })
    .await
    {
        Ok(lib) => lib,
        Err(_) => {
            return (
                [
                    (header::CONTENT_TYPE, "text/plain; charset=utf-8"),
                    (header::CACHE_CONTROL, "no-cache"),
                ],
                String::from("error"),
            );
        }
    };
    state.lib.store(Arc::new(new_lib));
    (
        [
            (header::CONTENT_TYPE, "text/plain; charset=utf-8"),
            (header::CACHE_CONTROL, "no-cache"),
        ],
        String::from("ok"),
    )
}
