mod library;
mod playlist;
mod server;

use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;

use axum::Router;
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

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Serve { path, port, bind, .. } => {
            let root = path;
            let lib = Arc::new(library::Library::scan(root.clone()));
            let bind = bind.unwrap_or_else(|| "127.0.0.1".parse().unwrap());
            let port = port.unwrap_or(8080);
            let base = format!("http://{bind}:{port}/");
            let state = server::AppState { lib: lib.clone(), base: base.clone(), root: root.clone() };
            let app: Router = server::build_router(state);
            let addr = SocketAddr::new(bind, port);
            let display_host = if bind.is_unspecified() {
                match local_ip_address::local_ip() {
                    Ok(std::net::IpAddr::V4(v4)) => v4.to_string(),
                    Ok(ip) => ip.to_string(),
                    Err(_) => bind.to_string(),
                }
            } else {
                bind.to_string()
            };
            let display_base = format!("http://{}:{}/", display_host, port);
            println!("root: {}", root.display());
            println!("listen: {}", &display_base.trim_end_matches('/'));
            println!("files: {}", lib.tracks().len());
            for a in lib.albums() {
                let enc = playlist::encode_path(&a.name);
                println!("album: {}album/{}.m3u8", display_base, enc);
            }
            println!("playlist: {}library.m3u8", display_base);
            println!("ui: {}", display_base.trim_end_matches('/'));
            let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
            axum::serve(listener, app).await.unwrap();
        }
    }
}
