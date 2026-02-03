//! gz-claude: TUI for orchestrating Zellij workspaces with Claude Code.
//!
//! @author waabox(waabox[at]gmail[dot]com)

mod cli;
mod config;
mod error;
mod git;
mod tui;
mod zellij;

use clap::Parser;
use cli::{Cli, Command};

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Panel) => {
            println!("Running in panel mode (inside Zellij)");
        }
        None => {
            println!("Starting gz-claude...");
        }
    }
}
