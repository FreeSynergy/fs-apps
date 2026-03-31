#![deny(clippy::all, clippy::pedantic, warnings)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::ignored_unit_patterns)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::struct_excessive_bools)]
//! `fs-theme-app` — Theme Manager for `FreeSynergy`.
//!
//! Without arguments, launches the desktop UI (Dioxus).
//! With arguments, runs the CLI or daemon.

use clap::Parser as _;
use tracing_subscriber::{fmt, EnvFilter};

use fs_theme_app::{
    controller::ThemeController,
    grpc::{GrpcThemeApp, ThemeAppServiceServer},
    rest,
    theme_cli::{Cli, Command},
};

// ── Entry point ───────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    fmt().with_env_filter(EnvFilter::from_default_env()).init();

    // If no args provided, launch the Dioxus desktop UI.
    let raw: Vec<String> = std::env::args().collect();
    if raw.len() == 1 {
        dioxus::launch(fs_theme_app::ThemeManagerApp);
        return Ok(());
    }

    let args = Cli::parse();
    let ctrl = ThemeController::new();

    match args.command {
        Command::Daemon => run_daemon(ctrl).await?,
        ref cmd => run_cli(cmd, &ctrl),
    }
    Ok(())
}

// ── Daemon ────────────────────────────────────────────────────────────────────

async fn run_daemon(ctrl: ThemeController) -> Result<(), Box<dyn std::error::Error>> {
    let grpc_port: u16 = std::env::var("FS_GRPC_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(50_090);
    let rest_port: u16 = std::env::var("FS_REST_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8_090);

    let grpc_addr: std::net::SocketAddr = ([0, 0, 0, 0], grpc_port).into();
    let rest_addr: std::net::SocketAddr = ([0, 0, 0, 0], rest_port).into();

    tracing::info!("gRPC on {grpc_addr}, REST on {rest_addr}");

    let grpc_ctrl = ctrl.clone();
    let grpc_task = tokio::spawn(async move {
        tonic::transport::Server::builder()
            .add_service(ThemeAppServiceServer::new(GrpcThemeApp::new(grpc_ctrl)))
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

fn run_cli(cmd: &Command, ctrl: &ThemeController) {
    match cmd {
        Command::Daemon => unreachable!(),
        Command::List => {
            for t in ctrl.list() {
                println!("{:30} v{}", t.name, t.version);
            }
        }
        Command::Active => {
            let t = ctrl.active();
            println!("{} v{}", t.name, t.version);
        }
        Command::Activate { name } => {
            if let Err(e) = ctrl.activate(name) {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
            println!("Activated: {name}");
        }
        Command::Preview { name } => {
            if let Some(css) = ctrl.preview_css(name) {
                println!("{css}");
            } else {
                eprintln!("Theme not found: {name}");
                std::process::exit(1);
            }
        }
    }
}
