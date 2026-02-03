//! CLI argument parsing for gz-claude.
//!
//! @author waabox(waabox[at]gmail[dot]com)

use clap::{Parser, Subcommand};

/// TUI for orchestrating Zellij workspaces with Claude Code.
#[derive(Parser, Debug)]
#[command(name = "gz-claude")]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Force enable web client
    #[arg(long)]
    pub web: bool,

    /// Force disable web client
    #[arg(long, conflicts_with = "web")]
    pub no_web: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Run the TUI panel (inside Zellij)
    Panel,
    /// Run the top bar (inside Zellij)
    TopBar,
}
