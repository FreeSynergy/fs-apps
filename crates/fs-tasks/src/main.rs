//! `fs-tasks` — FreeSynergy Task Pipelines daemon + CLI.
//!
//! | Variable       | Default |
//! |----------------|---------|
//! | `FS_GRPC_PORT` | `50094` |
//! | `FS_REST_PORT` | `8094`  |

#![deny(clippy::all, clippy::pedantic, warnings)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::ignored_unit_patterns)]
#![allow(clippy::needless_pass_by_value)]

use clap::Parser as _;
use tracing_subscriber::{fmt, EnvFilter};

use fs_tasks::{
    cli::{Cli, Command},
    controller::TaskController,
    grpc::{GrpcTasksApp, TasksServiceServer},
    rest,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    fmt().with_env_filter(EnvFilter::from_default_env()).init();

    let args = Cli::parse();
    let ctrl = TaskController::new();

    match args.command {
        Command::Daemon => run_daemon(ctrl).await?,
        ref cmd => run_cli(cmd, &ctrl),
    }
    Ok(())
}

async fn run_daemon(ctrl: TaskController) -> Result<(), Box<dyn std::error::Error>> {
    let grpc_port: u16 = std::env::var("FS_GRPC_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(50_093);
    let rest_port: u16 = std::env::var("FS_REST_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8_093);

    let grpc_addr: std::net::SocketAddr = ([0, 0, 0, 0], grpc_port).into();
    let rest_addr: std::net::SocketAddr = ([0, 0, 0, 0], rest_port).into();

    tracing::info!("gRPC on {grpc_addr}, REST on {rest_addr}");

    let grpc_ctrl = ctrl.clone();
    let grpc_task = tokio::spawn(async move {
        tonic::transport::Server::builder()
            .add_service(TasksServiceServer::new(GrpcTasksApp::new(grpc_ctrl)))
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

fn run_cli(cmd: &Command, ctrl: &TaskController) {
    match cmd {
        Command::Daemon => unreachable!(),
        Command::List => {
            let tasks = ctrl.list();
            if tasks.is_empty() {
                println!("No tasks configured.");
            } else {
                for t in &tasks {
                    println!(
                        "{:20}  {}  [{}]  {} → {}",
                        t.id,
                        t.name,
                        if t.enabled { "on " } else { "off" },
                        t.source.service,
                        t.target.service,
                    );
                }
            }
        }
        Command::Create { name } => {
            let task = ctrl.create(name.clone());
            println!("Created task {} ({})", task.id, task.name);
        }
        Command::Delete { id } => {
            if ctrl.delete(id) {
                println!("Deleted task {id}");
            } else {
                eprintln!("Task {id} not found");
                std::process::exit(1);
            }
        }
        Command::Toggle { id } => match ctrl.toggle(id) {
            Some(true) => println!("Task {id} enabled"),
            Some(false) => println!("Task {id} disabled"),
            None => {
                eprintln!("Task {id} not found");
                std::process::exit(1);
            }
        },
    }
}
