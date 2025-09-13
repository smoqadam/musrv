mod library;

use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;

use axum::{extract::State, http::header, routing::get, Router};
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
                .nest_service("/", files)
                .with_state(state);
            let addr = SocketAddr::new(bind, port);
            println!("root: {}", root.display());
            println!("listen: http://{}:{}", bind, port);
            println!("files: {}", lib.files().len());
            println!("playlist: {}library.m3u8", base);
            let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
            axum::serve(listener, app).await.unwrap();
        }
    }
}

async fn library_m3u8(State(state): State<AppState>) -> impl axum::response::IntoResponse {
    let mut body = String::from("#EXTM3U\n");
    for f in state.lib.files() {
        let rel = f.strip_prefix(&state.root).unwrap_or(f);
        let name = rel.file_name().and_then(|s| s.to_str()).unwrap_or("");
        let p = rel.to_string_lossy().replace('\\', "/");
        let encoded = p.split('/')
            .map(|s| urlencoding::encode(s).into_owned())
            .collect::<Vec<_>>()
            .join("/");
        body.push_str(&format!("#EXTINF:-1,{}\n{}{}\n", name, state.base, encoded));
    }
    ([
        (header::CONTENT_TYPE, "audio/x-mpegurl; charset=utf-8"),
        (header::CACHE_CONTROL, "no-cache"),
    ], body)
}
