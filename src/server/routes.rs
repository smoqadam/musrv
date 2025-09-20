use axum::Json;
use axum::http::Request;
use axum::{
    Router,
    extract::{Path as AxPath, Query, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};

use bytes::Bytes;

use std::sync::Arc;
use tower::util::ServiceExt;
use tower_http::{services::ServeFile, trace::TraceLayer};

use super::{helpers, state::AppState};

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/app.css", get(app_css))
        .route("/app.js", get(app_js))
        .route("/manifest.webmanifest", get(manifest))
        .route("/icon.svg", get(app_icon))
        .route("/api/folder", get(api_folder))
        .route("/api/folder.m3u8", get(api_folder_m3u8))
        .route("/api/artwork/:id", get(api_artwork))
        .route("/admin/rescan", get(admin_rescan))
        .route("/*path", get(static_file))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

fn static_asset(
    content_type: &'static str,
    body: &'static str,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    Ok((
        [
            (header::CONTENT_TYPE, content_type),
            (header::CACHE_CONTROL, "no-cache"),
        ],
        body.to_string(),
    ))
}

async fn index(State(_state): State<AppState>) -> Result<impl IntoResponse, (StatusCode, String)> {
    static_asset(
        "text/html; charset=utf-8",
        include_str!("../static/index.html"),
    )
}

async fn app_css(
    State(_state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    static_asset("text/css; charset=utf-8", include_str!("../static/app.css"))
}

async fn app_js(State(_state): State<AppState>) -> Result<impl IntoResponse, (StatusCode, String)> {
    static_asset(
        "application/javascript; charset=utf-8",
        include_str!("../static/app.js"),
    )
}

async fn manifest(
    State(_state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    static_asset(
        "application/manifest+json; charset=utf-8",
        include_str!("../static/manifest.webmanifest"),
    )
}

async fn app_icon(
    State(_state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    static_asset("image/svg+xml", include_str!("../static/icon.svg"))
}

#[derive(serde::Deserialize)]
struct FolderQuery {
    path: Option<String>,
}

use super::types::{JsonFolderAlbum, JsonFolderResp, JsonFolderTrack};

async fn api_folder(
    Query(q): Query<FolderQuery>,
    State(state): State<AppState>,
) -> Result<Json<JsonFolderResp>, (StatusCode, String)> {
    let rel = match q.path {
        Some(path) if !path.is_empty() => helpers::validate_request_path(&path)
            .map_err(|_| (StatusCode::BAD_REQUEST, String::new()))?,
        _ => String::new(),
    };
    let name = if rel.is_empty() {
        "/".to_string()
    } else {
        rel.rsplit('/').next().unwrap_or("").to_string()
    };
    let mut albums = Vec::new();
    let lib = state.lib.load();
    if let Some(entry) = lib.folder(&rel) {
        for child in &entry.subfolders {
            let child_name = child.rsplit('/').next().unwrap_or("").to_string();
            albums.push(JsonFolderAlbum {
                name: child_name,
                path: child.clone(),
            });
        }
    }
    let base_url = state.base.clone();
    let base_trimmed = state.base.trim_end_matches('/').to_string();
    let tracks = lib
        .collect_tracks_recursive(&rel)
        .into_iter()
        .map(|track| {
            let rel_path = track.path.to_string_lossy().replace('\\', "/");
            let file_name = track
                .path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            let metadata = &track.metadata;
            let display_name = metadata.title.clone().unwrap_or_else(|| file_name.clone());
            let encoded = crate::playlist::encode_path(&rel_path);
            let artwork_url = metadata
                .artwork_id
                .as_ref()
                .map(|id| format!("{base_trimmed}/api/artwork/{id}"));
            JsonFolderTrack {
                name: file_name,
                display_name,
                relative_path: rel_path,
                url: format!("{base_url}{encoded}"),
                title: metadata.title.clone(),
                artist: metadata.artist.clone(),
                album: metadata.album.clone(),
                duration: metadata.duration,
                artwork_url,
            }
        })
        .collect();
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
        tracks,
    };
    Ok(Json(body))
}

async fn api_folder_m3u8(
    Query(q): Query<FolderQuery>,
    State(state): State<AppState>,
) -> Result<impl axum::response::IntoResponse, (StatusCode, String)> {
    let rel = match q.path {
        Some(path) if !path.is_empty() => helpers::validate_request_path(&path)
            .map_err(|_| (StatusCode::BAD_REQUEST, String::new()))?,
        _ => String::new(),
    };
    let lib = state.lib.load();
    let tracks = lib.collect_tracks_recursive(&rel);
    let body = crate::playlist::render_m3u8(&state.base, &state.root, &tracks);
    Ok((
        [
            (header::CONTENT_TYPE, "audio/x-mpegurl; charset=utf-8"),
            (header::CACHE_CONTROL, "no-cache"),
        ],
        body,
    ))
}

async fn api_artwork(
    AxPath(id): AxPath<String>,
    State(state): State<AppState>,
) -> Result<Response, (StatusCode, String)> {
    let lib = state.lib.load();
    let Some(art) = lib.artwork(&id) else {
        return Err((StatusCode::NOT_FOUND, String::new()));
    };

    let data = Bytes::copy_from_slice(&art.data);
    let mut response = Response::new(axum::body::Body::from(data));
    let content_type = header::HeaderValue::from_str(&art.mime)
        .unwrap_or_else(|_| header::HeaderValue::from_static("image/jpeg"));
    response
        .headers_mut()
        .insert(header::CONTENT_TYPE, content_type);
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        header::HeaderValue::from_static("public, max-age=86400"),
    );
    Ok(response)
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
