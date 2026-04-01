// cli.rs — CLI for fs-container-app.

use clap::{Parser, Subcommand};

/// `FreeSynergy` Container App — manage container services.
#[derive(Parser)]
#[command(
    name = "fs-container",
    version,
    about = "FreeSynergy container service management"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Run as daemon (gRPC + REST server).
    Daemon,
    /// List all managed container services.
    List,
    /// Start a container service.
    Start {
        /// Service name to start.
        name: String,
    },
    /// Stop a container service.
    Stop {
        /// Service name to stop.
        name: String,
    },
    /// Show recent log lines for a service.
    Logs {
        /// Service name.
        name: String,
        /// Number of lines to show.
        #[arg(short, long, default_value = "50")]
        lines: usize,
    },
}
