// cli.rs — CLI for fs-bots.

use clap::{Parser, Subcommand};

/// `FreeSynergy` Bot Manager daemon and CLI.
#[derive(Parser)]
#[command(name = "fs-bots", version, about = "Manage FreeSynergy messaging bots")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Run as daemon (gRPC + REST server).
    Daemon,
    /// List all bots.
    List,
    /// Enable a bot by id.
    Enable {
        /// Bot id.
        id: String,
    },
    /// Disable a bot by id.
    Disable {
        /// Bot id.
        id: String,
    },
}
