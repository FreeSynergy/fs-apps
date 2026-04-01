#![deny(clippy::all, clippy::pedantic, warnings)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::ignored_unit_patterns)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::struct_excessive_bools)]
//! `fs-lenses` — FreeSynergy Lenses daemon and CLI.
//!
//! # Environment variables
//!
//! | Variable       | Default |
//! |----------------|---------|
//! | `FS_GRPC_PORT` | `50091` |
//! | `FS_REST_PORT` | `8091`  |

use clap::Parser as _;
use tracing_subscriber::{fmt, EnvFilter};

use fs_lenses::{
    cli::{Cli, Command},
    controller::LensController,
    grpc::{GrpcLensApp, LensServiceServer},
    rest,
};

// ── Entry point ───────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    fmt().with_env_filter(EnvFilter::from_default_env()).init();

    let args = Cli::parse();
    let ctrl = LensController::new();

    match args.command {
        Command::Daemon => run_daemon(ctrl).await?,
        ref cmd => run_cli(cmd, &ctrl),
    }
    Ok(())
}

// ── Daemon ────────────────────────────────────────────────────────────────────

async fn run_daemon(ctrl: LensController) -> Result<(), Box<dyn std::error::Error>> {
    let grpc_port: u16 = std::env::var("FS_GRPC_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(50_091);
    let rest_port: u16 = std::env::var("FS_REST_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8_091);

    let grpc_addr: std::net::SocketAddr = ([0, 0, 0, 0], grpc_port).into();
    let rest_addr: std::net::SocketAddr = ([0, 0, 0, 0], rest_port).into();

    tracing::info!("gRPC on {grpc_addr}, REST on {rest_addr}");

    let grpc_ctrl = ctrl.clone();
    let grpc_task = tokio::spawn(async move {
        tonic::transport::Server::builder()
            .add_service(LensServiceServer::new(GrpcLensApp::new(grpc_ctrl)))
            .serve(grpc_addr)
            .await
            .unwrap();
    });

    let rest_task = tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind(rest_addr).await.unwrap();
        axum::serve(listener, rest::router(ctrl)).await.unwrap();
    });

    tokio::try_join!(grpc_task, rest_task)?;
    Ok(())
}

// ── CLI ───────────────────────────────────────────────────────────────────────

fn run_cli(cmd: &Command, ctrl: &LensController) {
    match cmd {
        Command::Daemon => unreachable!(),
        Command::List => {
            for l in ctrl.list() {
                println!("{:5}  {:30}  {}", l.id, l.name, l.query);
            }
        }
        Command::Create { name, query } => {
            let l = ctrl.create(name.clone(), query.clone());
            println!("Created lens #{}: {}", l.id, l.name);
        }
        Command::Delete { id } => {
            if ctrl.delete(*id) {
                println!("Deleted lens #{id}");
            } else {
                eprintln!("Lens #{id} not found");
                std::process::exit(1);
            }
        }
        Command::Query { lens_id, query } => {
            let results = ctrl.query(*lens_id, query);
            if results.is_empty() {
                println!("(no results)");
            } else {
                for r in results {
                    println!("{r}");
                }
            }
        }
    }
}
