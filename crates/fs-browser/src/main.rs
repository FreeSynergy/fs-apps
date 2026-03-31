#![deny(clippy::all, clippy::pedantic, warnings)]
//! `fs-browser` — `FreeSynergy` web browser daemon and CLI.
//!
//! # Environment variables
//!
//! | Variable              | Default                 |
//! |-----------------------|-------------------------|
//! | `FS_GRPC_PORT`        | `50070`                 |
//! | `FS_REST_PORT`        | `8080`                  |

use std::{net::SocketAddr, sync::Arc};

use clap::Parser as _;
use tonic::transport::Server as GrpcServer;
use tracing::info;
use tracing_subscriber::{fmt, EnvFilter};

use fs_browser::{
    bookmark::InMemoryBookmarkStore,
    cli::{BookmarkAction, Cli, Command},
    controller::BrowserController,
    grpc::{BrowserServiceServer, GrpcBrowser},
    rest,
};

// ── Config ────────────────────────────────────────────────────────────────────

struct Config {
    grpc_addr: SocketAddr,
    rest_addr: SocketAddr,
}

impl Config {
    fn from_env() -> Self {
        let grpc_port: u16 = std::env::var("FS_GRPC_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(50_070);
        let rest_port: u16 = std::env::var("FS_REST_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8_080);
        Self {
            grpc_addr: SocketAddr::from(([0, 0, 0, 0], grpc_port)),
            rest_addr: SocketAddr::from(([0, 0, 0, 0], rest_port)),
        }
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    fmt().with_env_filter(EnvFilter::from_default_env()).init();

    let args = Cli::parse();
    let cfg = Config::from_env();

    match args.command {
        Command::Daemon => run_daemon(cfg).await?,
        cmd => run_cli(cmd).await?,
    }
    Ok(())
}

// ── Daemon ────────────────────────────────────────────────────────────────────

async fn run_daemon(cfg: Config) -> Result<(), Box<dyn std::error::Error>> {
    let store = Arc::new(InMemoryBookmarkStore::new());
    let ctrl = Arc::new(BrowserController::new(Arc::clone(&store)));

    info!("gRPC listening on {}", cfg.grpc_addr);
    info!("REST listening on {}", cfg.rest_addr);

    let rest_ctrl = Arc::clone(&ctrl);
    tokio::spawn(async move {
        let app = rest::router(rest_ctrl);
        let listener = tokio::net::TcpListener::bind(cfg.rest_addr).await.unwrap();
        axum::serve(listener, app).await.unwrap();
    });

    GrpcServer::builder()
        .add_service(BrowserServiceServer::new(GrpcBrowser::new(Arc::clone(
            &ctrl,
        ))))
        .serve(cfg.grpc_addr)
        .await?;

    Ok(())
}

// ── CLI ───────────────────────────────────────────────────────────────────────

async fn run_cli(cmd: Command) -> Result<(), Box<dyn std::error::Error>> {
    let ctrl = BrowserController::new(Arc::new(InMemoryBookmarkStore::new()));

    match cmd {
        Command::Daemon => unreachable!(),
        Command::Open { url } => {
            ctrl.open_url(&url);
            println!("Opened: {url}");
        }
        Command::History => {
            let hist = ctrl.history();
            if hist.is_empty() {
                println!("No history.");
            } else {
                for e in hist {
                    println!("{} — {}", e.visited_at, e.url);
                }
            }
        }
        Command::Bookmarks {
            action: BookmarkAction::List,
        } => {
            let list = ctrl.list_bookmarks().await;
            if list.is_empty() {
                println!("No bookmarks.");
            } else {
                for b in list {
                    println!("[{}] {} — {}", b.id, b.title, b.url);
                }
            }
        }
        Command::Bookmarks {
            action: BookmarkAction::Add { title, url },
        } => {
            let b = ctrl.add_bookmark(&title, &url).await;
            println!("Bookmark added: [{}] {} — {}", b.id, b.title, b.url);
        }
        Command::Bookmarks {
            action: BookmarkAction::Remove { id },
        } => {
            if ctrl.remove_bookmark(&id).await {
                println!("Bookmark {id} removed.");
            } else {
                println!("Bookmark {id} not found.");
            }
        }
    }
    Ok(())
}
