mod library;
mod playlist;
mod server;

use std::net::{IpAddr, SocketAddr};
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use axum::Router;
use clap::{ArgAction, Parser, Subcommand};
use tracing_subscriber::{EnvFilter, fmt};

#[derive(Parser)]
#[command(
    name = "musrv",
    author,
    version,
    about = "Minimal, zero‑config music server that scans a folder, serves a small web UI, M3U8 playlists and a simple radio stream.",
    long_about = None
)]
struct Cli {
    #[arg(short, long, action = ArgAction::Count, global = true)]
    verbose: u8,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Serve a music folder over HTTP
    Serve {
        /// Root directory to scan and serve
        #[arg(value_name = "ROOT", value_hint = clap::ValueHint::DirPath)]
        path: PathBuf,

        /// TCP port to listen on (default: 8080)
        #[arg(long, value_name = "PORT")]
        port: Option<u16>,

        /// IP address to bind (e.g. 127.0.0.1, 0.0.0.0)
        #[arg(long, value_name = "IP")]
        bind: Option<IpAddr>,

        /// Reserved for future ignore rules
        #[arg(long, hide = false)]
        gitignore: bool,

        /// Album grouping depth (0 = full parent path, 1 = top folder)
        #[arg(long, value_name = "N", value_parser = clap::value_parser!(usize))]
        album_depth: Option<usize>,

        /// Print a QR code for the UI URL
        #[arg(long)]
        qr: bool,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let level = match cli.verbose {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));
    fmt().with_env_filter(filter).init();
    match cli.command {
        Commands::Serve {
            path,
            port,
            bind,
            album_depth,
            qr,
            ..
        } => {
            if !Path::new(&path).exists() {
                anyhow::bail!("path does not exist: {}", path.display());
            }
            if !std::fs::metadata(&path)
                .map(|m| m.is_dir())
                .unwrap_or(false)
            {
                anyhow::bail!("path is not a directory: {}", path.display());
            }
            let root = std::fs::canonicalize(&path).unwrap_or(path);
            let depth = album_depth.unwrap_or(1);
            let lib = Arc::new(library::Library::scan_with_depth(root.clone(), depth));
            let bind = bind.unwrap_or_else(|| "127.0.0.1".parse().unwrap());
            let port = port.unwrap_or(8080);
            let display_host = if bind.is_unspecified() {
                match local_ip_address::local_ip() {
                    Ok(std::net::IpAddr::V4(v4)) => v4.to_string(),
                    Ok(ip) => ip.to_string(),
                    Err(_) => bind.to_string(),
                }
            } else {
                bind.to_string()
            };
            let base = format!("http://{display_host}:{port}/");
            let state = server::AppState {
                lib: Arc::new(arc_swap::ArcSwap::from(lib.clone())),
                base: base.clone(),
                root: root.clone(),
                album_depth: depth,
            };
            let app: Router = server::build_router(state);
            let addr = SocketAddr::new(bind, port);
            println!("root: {}", root.display());
            println!("listen: {}", &base.trim_end_matches('/'));
            println!(
                "tracks: {} | albums: {}",
                lib.tracks().len(),
                lib.albums().len()
            );
            println!("ui: {}", base.trim_end_matches('/'));
            println!("library.m3u8: {base}library.m3u8");
            if qr {
                let ui_url = base.trim_end_matches('/');
                if let Ok(code) = qrcode::QrCode::new(ui_url.as_bytes()) {
                    use qrcode::render::unicode;
                    let qr_art = code.render::<unicode::Dense1x2>().quiet_zone(true).build();
                    println!("\nscan to open UI:\n{qr_art}");
                }
            }
            let listener = tokio::net::TcpListener::bind(addr).await?;
            axum::serve(listener, app).await?;
        }
    }
    Ok(())
}
