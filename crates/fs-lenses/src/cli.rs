// cli.rs — CLI for fs-lenses.

use clap::{Parser, Subcommand};

/// `FreeSynergy` Lenses CLI.
#[derive(Parser)]
#[command(
    name = "fs-lenses",
    version,
    about = "Manage FreeSynergy Lenses (list, create, delete, query)"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Run as daemon (gRPC + REST server).
    Daemon,
    /// List all saved lenses.
    List,
    /// Create a new lens.
    Create {
        /// Name for the new lens.
        name: String,
        /// Search query for the lens.
        query: String,
    },
    /// Delete a lens by id.
    Delete {
        /// Lens id to delete.
        id: i64,
    },
    /// Run a query through a lens.
    Query {
        /// Lens id to query.
        lens_id: i64,
        /// Search query string.
        query: String,
    },
}
