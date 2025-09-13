mod library;
mod playlist;

use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;

use axum::{extract::{Path as AxPath, State}, http::header, routing::get, Router};
use tower_http::services::ServeDir;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "musrv")]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Serve {
        path: PathBuf,
        #[arg(long)]
        port: Option<u16>,
        #[arg(long)]
        bind: Option<IpAddr>,
        #[arg(long)]
        gitignore: bool,
    },
}

#[derive(Clone)]
struct AppState {
    lib: Arc<library::Library>,
    base: String,
    root: PathBuf,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Serve { path, port, bind, .. } => {
            let root = path;
            let lib = Arc::new(library::Library::scan(root.clone()));
            let bind = bind.unwrap_or_else(|| "127.0.0.1".parse().unwrap());
            let port = port.unwrap_or(8080);
            let base = format!("http://{}:{}/", bind, port);
            let state = AppState { lib: lib.clone(), base: base.clone(), root: root.clone() };
            let files = ServeDir::new(root.clone());
            let app = Router::new()
                .route("/library.m3u8", get(library_m3u8))
                .route("/album/*name", get(album_m3u8))
                .nest_service("/", files)
                .with_state(state);
            let addr = SocketAddr::new(bind, port);
            println!("root: {}", root.display());
            println!("listen: http://{}:{}", bind, port);
            println!("files: {}", lib.tracks().len());
            for a in lib.albums() {
                println!("album: {}album/{}.m3u8", base, a.name);
            }
            println!("playlist: {}library.m3u8", base);
            let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
            axum::serve(listener, app).await.unwrap();
        }
    }
}

async fn library_m3u8(State(state): State<AppState>) -> impl axum::response::IntoResponse {
    let body = playlist::render_m3u8(&state.base, &state.root, state.lib.tracks());
    ([
        (header::CONTENT_TYPE, "audio/x-mpegurl; charset=utf-8"),
        (header::CACHE_CONTROL, "no-cache"),
    ], body)
}

async fn album_m3u8(AxPath(mut name): AxPath<String>, State(state): State<AppState>) -> impl axum::response::IntoResponse {
    if let Some(stripped) = name.strip_suffix(".m3u8") { name = stripped.to_string(); }
    if let Ok(decoded) = urlencoding::decode(&name) { name = decoded.into_owned(); }
    if let Some(album) = state.lib.album_by_name(&name) {
        let body = playlist::render_m3u8(&state.base, &state.root, &album.tracks);
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
