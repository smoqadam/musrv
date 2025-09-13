use std::sync::Arc;

use axum::{body, body::Body, http::Request};
use tower::util::ServiceExt;

fn write_file(path: &std::path::Path) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, b"").unwrap();
}

#[tokio::test]
async fn library_json_and_playlists() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().to_path_buf();
    write_file(&root.join("Album1/song1.mp3"));
    write_file(&root.join("Album1/song2.flac"));
    write_file(&root.join("loose.mp3"));

    let lib = musrv::library::Library::scan(root.clone());
    let state = musrv::server::AppState {
        lib: Arc::new(arc_swap::ArcSwap::from(Arc::new(lib))),
        base: "http://127.0.0.1:9999/".to_string(),
        root: root.clone(),
        album_depth: 1,
    };
    let app = musrv::server::build_router(state);

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/library.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(res.status().is_success());
    let bytes = body::to_bytes(res.into_body(), 1024 * 1024).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(v.get("albums").is_some());

    let res2 = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/library.m3u8")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(res2.status().is_success());
    let ctype = res2
        .headers()
        .get(axum::http::header::CONTENT_TYPE)
        .unwrap();
    assert!(ctype.to_str().unwrap().starts_with("audio/x-mpegurl"));

    let res3 = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/album/Album1.m3u8")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(res3.status().is_success());
}
