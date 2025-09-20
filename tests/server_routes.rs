use std::sync::Arc;

use axum::{body, body::Body, http::{Request, StatusCode}};
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
    };
    let app = musrv::server::build_router(state);

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/folder")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(res.status().is_success());
    let bytes = body::to_bytes(res.into_body(), 1024 * 1024).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(v.get("albums").is_some());
    assert!(v.get("tracks").is_some());

    let res2 = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/folder.m3u8?path=Album1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(res2.status().is_success());

    let res3 = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/artwork/does-not-exist")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res3.status(), StatusCode::NOT_FOUND);
}
