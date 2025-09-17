use axum::Json;
use axum::http::Request;
use axum::{
    Router,
    extract::{Path as AxPath, Query, State},
    http::{StatusCode, header},
    response::IntoResponse,
    routing::get,
};

use std::sync::Arc;
use tower::util::ServiceExt;
use tower_http::{services::ServeFile, trace::TraceLayer};

use super::{helpers, state::AppState};

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/api/folder", get(api_folder))
        .route("/api/folder.m3u8", get(api_folder_m3u8))
        .route("/admin/rescan", get(admin_rescan))
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

#[derive(serde::Deserialize)]
struct FolderQuery {
    path: Option<String>,
}

use super::types::{JsonFolderAlbum, JsonFolderResp};

async fn api_folder(
    Query(q): Query<FolderQuery>,
    State(state): State<AppState>,
) -> impl axum::response::IntoResponse {
    let rel = q.path.unwrap_or_default();
    let rel = if rel.is_empty() {
        String::new()
    } else {
        helpers::validate_request_path(&rel).unwrap_or_default()
    };
    let name = if rel.is_empty() {
        "/".to_string()
    } else {
        rel.rsplit('/').next().unwrap_or("").to_string()
    };
    let mut albums = Vec::new();
    if let Some(entry) = state.lib.load().folder(&rel) {
        for child in &entry.subfolders {
            let child_name = child.rsplit('/').next().unwrap_or("").to_string();
            albums.push(JsonFolderAlbum {
                name: child_name,
                path: child.clone(),
            });
        }
    }
    let m3u8 = format!(
        "{}/api/folder.m3u8?path={}",
        state.base.trim_end_matches('/'),
        urlencoding::encode(&rel)
    );
    let body = JsonFolderResp {
        name,
        path: rel,
        m3u8,
        albums,
    };
    Json(body)
}

async fn api_folder_m3u8(
    Query(q): Query<FolderQuery>,
    State(state): State<AppState>,
) -> impl axum::response::IntoResponse {
    let rel = q.path.unwrap_or_default();
    let rel = if rel.is_empty() {
        String::new()
    } else {
        helpers::validate_request_path(&rel).unwrap_or_default()
    };
    let tracks = state.lib.load().collect_tracks_recursive(&rel);
    let body = crate::playlist::render_m3u8(&state.base, &state.root, &tracks);
    (
        [
            (header::CONTENT_TYPE, "audio/x-mpegurl; charset=utf-8"),
            (header::CACHE_CONTROL, "no-cache"),
        ],
        body,
    )
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
    let new_lib =
        match tokio::task::spawn_blocking(move || crate::library::Library::scan(root)).await {
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
