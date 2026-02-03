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
use config::Config;

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Panel) => {
            run_panel();
        }
        None => {
            run_main(cli.web, cli.no_web);
        }
    }
}

fn run_main(force_web: bool, force_no_web: bool) {
    // Check if Zellij is installed
    if !zellij::is_zellij_installed() {
        eprintln!(
            "Error: Zellij not found\n\n\
             gz-claude requires Zellij to be installed.\n\
             Install it from: https://zellij.dev/documentation/installation"
        );
        std::process::exit(1);
    }

    // Load configuration
    let config = match Config::load() {
        Ok(config) => config,
        Err(e) => {
            if matches!(
                e,
                error::GzClaudeError::Config(error::ConfigError::NotFound(_))
            ) {
                // Create example config
                match Config::create_example() {
                    Ok(path) => {
                        println!(
                            "Created example configuration at {}\n\
                             Please edit it to add your workspaces and run again.",
                            path.display()
                        );
                    }
                    Err(e) => {
                        eprintln!("Error creating example config: {}", e);
                    }
                }
            } else {
                eprintln!("Error loading configuration: {}", e);
            }
            std::process::exit(1);
        }
    };

    // Validate configuration
    if let Err(e) = config.validate() {
        eprintln!("Error: Invalid configuration\n\n{}", e);
        eprintln!(
            "\nPlease fix the configuration at {}",
            Config::default_path().display()
        );
        std::process::exit(1);
    }

    // Determine web client behavior (for Etapa 5)
    let _start_web = if force_web {
        true
    } else if force_no_web {
        false
    } else {
        config.web_client.auto_start
    };

    // Generate the Zellij layout
    if let Err(e) = zellij::generate_layout() {
        eprintln!("Error generating Zellij layout: {}", e);
        std::process::exit(1);
    }

    // Start Zellij with the gz-claude layout
    if let Err(e) = zellij::start_zellij() {
        eprintln!("Error starting Zellij: {}", e);
        std::process::exit(1);
    }
}

fn run_panel() {
    // Check if running inside Zellij
    if std::env::var("ZELLIJ").is_err() {
        eprintln!(
            "Error: gz-claude panel must be run inside Zellij.\n\
             Run 'gz-claude' without arguments to start Zellij with the proper layout."
        );
        std::process::exit(1);
    }

    // Load configuration
    let config = match Config::load() {
        Ok(config) => config,
        Err(e) => {
            if matches!(
                e,
                error::GzClaudeError::Config(error::ConfigError::NotFound(_))
            ) {
                // Create example config
                match Config::create_example() {
                    Ok(path) => {
                        eprintln!(
                            "Created example configuration at {}\n\
                             Please edit it to add your workspaces and run again.",
                            path.display()
                        );
                    }
                    Err(e) => {
                        eprintln!("Error creating example config: {}", e);
                    }
                }
            } else {
                eprintln!("Error loading configuration: {}", e);
            }
            std::process::exit(1);
        }
    };

    // Validate configuration
    if let Err(e) = config.validate() {
        eprintln!("Error: Invalid configuration\n\n{}", e);
        eprintln!(
            "\nPlease fix the configuration at {}",
            Config::default_path().display()
        );
        std::process::exit(1);
    }

    // Run the TUI
    if let Err(e) = tui::run(&config) {
        eprintln!("Error running TUI: {}", e);
        std::process::exit(1);
    }
}
