// cli.rs — CLI for fs-tasks.

use clap::{Parser, Subcommand};

/// `FreeSynergy` Tasks CLI.
#[derive(Parser)]
#[command(
    name = "fs-tasks",
    version,
    about = "Manage FreeSynergy Task Pipelines (list, create, delete, toggle)"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Run as daemon (gRPC + REST server).
    Daemon,
    /// List all task pipelines.
    List,
    /// Create a new task pipeline.
    Create {
        /// Name for the new task pipeline.
        name: String,
    },
    /// Delete a task pipeline by id.
    Delete {
        /// Task id to delete.
        id: String,
    },
    /// Toggle a task pipeline enabled/disabled.
    Toggle {
        /// Task id to toggle.
        id: String,
    },
}
