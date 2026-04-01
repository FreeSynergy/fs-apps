//! `fs-builder` — `FreeSynergy` package builder daemon + CLI.
//!
//! | Variable       | Default |
//! |----------------|---------|
//! | `FS_GRPC_PORT` | `50096` |
//! | `FS_REST_PORT` | `8096`  |

#![deny(clippy::all, clippy::pedantic, warnings)]
#![allow(clippy::must_use_candidate)]

use clap::Parser as _;
use tracing_subscriber::{fmt, EnvFilter};

use fs_builder::{
    cli::{Cli, Command},
    controller::BuilderController,
    grpc::{BuilderServiceServer, GrpcBuilderApp},
    rest,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    fmt().with_env_filter(EnvFilter::from_default_env()).init();

    let args = Cli::parse();
    let ctrl = BuilderController::new();

    match args.command {
        Command::Daemon => run_daemon(ctrl).await?,
        ref cmd => run_cli(cmd, &ctrl),
    }
    Ok(())
}

async fn run_daemon(ctrl: BuilderController) -> Result<(), Box<dyn std::error::Error>> {
    let grpc_port: u16 = std::env::var("FS_GRPC_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(50_096);
    let rest_port: u16 = std::env::var("FS_REST_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8_096);

    let grpc_addr: std::net::SocketAddr = ([0, 0, 0, 0], grpc_port).into();
    let rest_addr: std::net::SocketAddr = ([0, 0, 0, 0], rest_port).into();

    tracing::info!("gRPC on {grpc_addr}, REST on {rest_addr}");

    let grpc_ctrl = ctrl.clone();
    let grpc_task = tokio::spawn(async move {
        tonic::transport::Server::builder()
            .add_service(BuilderServiceServer::new(GrpcBuilderApp::new(grpc_ctrl)))
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

fn run_cli(cmd: &Command, ctrl: &BuilderController) {
    match cmd {
        Command::Daemon => unreachable!(),
        Command::List => {
            let ps = ctrl.list();
            if ps.is_empty() {
                println!("No active pipelines.");
            }
            for p in &ps {
                println!("{}", p.package_path);
            }
        }
        Command::Analyse { path }
        | Command::Validate { path }
        | Command::Build { path }
        | Command::Publish { path } => {
            let _p = ctrl.start(path);
            println!("Pipeline started for: {path}");
        }
    }
}
