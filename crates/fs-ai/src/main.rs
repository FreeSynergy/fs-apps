//! `fs-ai` — FreeSynergy AI assistant daemon and CLI.
//!
//! # Environment variables
//!
//! | Variable       | Default |
//! |----------------|---------|
//! | `FS_GRPC_PORT` | `50095` |
//! | `FS_REST_PORT` | `8095`  |

#![deny(clippy::all, clippy::pedantic, warnings)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::ignored_unit_patterns)]
#![allow(clippy::needless_pass_by_value)]

use clap::Parser as _;
use tracing_subscriber::{fmt, EnvFilter};

use fs_ai::{
    cli::{Cli, Command},
    controller::AiController,
    grpc::{AiServiceServer, GrpcAiApp},
    rest,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    fmt().with_env_filter(EnvFilter::from_default_env()).init();

    let args = Cli::parse();
    let ctrl = AiController::new();

    match args.command {
        Command::Daemon => run_daemon(ctrl).await?,
        ref cmd => run_cli(cmd, &ctrl),
    }
    Ok(())
}

async fn run_daemon(ctrl: AiController) -> Result<(), Box<dyn std::error::Error>> {
    let grpc_port: u16 = std::env::var("FS_GRPC_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(50_095);
    let rest_port: u16 = std::env::var("FS_REST_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8_095);

    let grpc_addr: std::net::SocketAddr = ([0, 0, 0, 0], grpc_port).into();
    let rest_addr: std::net::SocketAddr = ([0, 0, 0, 0], rest_port).into();

    tracing::info!("gRPC on {grpc_addr}, REST on {rest_addr}");

    let grpc_ctrl = ctrl.clone();
    let grpc_task = tokio::spawn(async move {
        tonic::transport::Server::builder()
            .add_service(AiServiceServer::new(GrpcAiApp::new(grpc_ctrl)))
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

fn run_cli(cmd: &Command, ctrl: &AiController) {
    match cmd {
        Command::Daemon => unreachable!(),
        Command::Models => {
            for m in ctrl.list_models() {
                println!("{:30}  {}", m.id, m.name);
            }
        }
        Command::Status => {
            let snap = ctrl.snapshot();
            if snap.running {
                println!(
                    "running (port {}, API: {})",
                    snap.port.unwrap_or(0),
                    snap.api_url().unwrap_or_default()
                );
            } else {
                println!("stopped");
            }
        }
        Command::Start { model } => match ctrl.start(model) {
            Ok(port) => println!("Engine started on port {port}"),
            Err(e) => {
                eprintln!("Failed to start: {e}");
                std::process::exit(1);
            }
        },
        Command::Stop => match ctrl.stop() {
            Ok(()) => println!("Engine stopped"),
            Err(e) => {
                eprintln!("Failed to stop: {e}");
                std::process::exit(1);
            }
        },
    }
}
