// cli.rs — CLI for fs-builder.

use clap::{Parser, Subcommand};

/// `FreeSynergy` package builder — analyse, validate, build, publish.
#[derive(Parser)]
#[command(name = "fs-builder", version, about = "FreeSynergy package builder")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Start the builder daemon (gRPC + REST).
    Daemon,
    /// Analyse a package directory.
    Analyse {
        /// Path to the package directory.
        path: String,
    },
    /// Validate a package directory.
    Validate {
        /// Path to the package directory.
        path: String,
    },
    /// Build a package.
    Build {
        /// Path to the package directory.
        path: String,
    },
    /// Build and publish a package to the Store.
    Publish {
        /// Path to the package directory.
        path: String,
    },
    /// List active build pipelines.
    List,
}
