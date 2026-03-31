// theme_cli.rs — CLI for fs-theme-app.

use clap::{Parser, Subcommand};

/// `FreeSynergy` Theme Manager CLI.
#[derive(Parser)]
#[command(
    name = "fs-theme-app",
    version,
    about = "Manage FreeSynergy themes (list, activate, preview)"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Run as daemon (gRPC + REST server).
    Daemon,
    /// List all available themes.
    List,
    /// Show the currently active theme.
    Active,
    /// Activate a theme by name.
    Activate {
        /// Name of the theme to activate.
        name: String,
    },
    /// Print CSS for a theme (preview).
    Preview {
        /// Name of the theme to preview.
        name: String,
    },
}
