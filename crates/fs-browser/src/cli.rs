// cli.rs — CLI for fs-browser.

use clap::{Parser, Subcommand};

/// `FreeSynergy` Browser — navigate the web from the command line.
#[derive(Parser)]
#[command(name = "fs-browser", version, about = "FreeSynergy web browser")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Run as daemon (gRPC server + REST API).
    Daemon,
    /// Open a URL.
    Open {
        /// URL to navigate to.
        url: String,
    },
    /// Show navigation history.
    History,
    /// Manage bookmarks.
    Bookmarks {
        #[command(subcommand)]
        action: BookmarkAction,
    },
}

#[derive(Subcommand)]
pub enum BookmarkAction {
    /// List all bookmarks.
    List,
    /// Add a bookmark.
    Add {
        /// Bookmark title.
        #[arg(short, long)]
        title: String,
        /// URL to bookmark.
        #[arg(short, long)]
        url: String,
    },
    /// Remove a bookmark by ID.
    Remove {
        /// Bookmark ID.
        id: String,
    },
}
