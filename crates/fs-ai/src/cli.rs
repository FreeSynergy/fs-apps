// cli.rs — CLI for fs-ai.

use clap::{Parser, Subcommand};

/// `FreeSynergy` AI assistant — manage the local LLM engine.
#[derive(Parser)]
#[command(
    name = "fs-ai",
    version,
    about = "FreeSynergy AI assistant daemon and CLI"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Run as daemon (gRPC + REST server).
    Daemon,
    /// List available AI models.
    Models,
    /// Show current engine status.
    Status,
    /// Start the LLM engine.
    Start {
        /// Model ID to start (e.g. qwen3-4b).
        model: String,
    },
    /// Stop the LLM engine.
    Stop,
}
