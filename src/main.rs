mod library;

use std::net::IpAddr;
use std::path::PathBuf;

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

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Serve { path, .. } => {
            let lib = library::Library::scan(path);
            println!("{:?}", lib);
        }
    }
}
