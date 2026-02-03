//! gz-claude: TUI for orchestrating Zellij workspaces with Claude Code.
//!
//! @author waabox(waabox[at]gmail[dot]com)

mod cli;
mod config;
mod error;
mod git;
mod session;
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
        Some(Command::TopBar) => {
            run_top_bar();
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

    // Determine web client behavior
    let start_web = if force_web {
        true
    } else if force_no_web {
        false
    } else {
        config.web_client.auto_start
    };

    // Clear any previous web URL
    let _ = zellij::clear_web_url();

    // Ensure SSL certificates exist for network access
    if let Err(e) = zellij::ensure_ssl_certs() {
        eprintln!("Warning: Failed to generate SSL certificates: {}", e);
    }

    // Start web server if enabled
    let _web_child = if start_web {
        match zellij::start_web_server(&config.web_client.bind_address, config.web_client.port) {
            Ok((child, use_ssl)) => {
                // Wait a moment for the server to be ready
                std::thread::sleep(std::time::Duration::from_millis(500));

                // Create a token after starting the server
                match zellij::create_web_token() {
                    Ok(token) => {
                        let url = zellij::web_url(config.web_client.port, &token, use_ssl);
                        // Save URL for top bar to display
                        if let Err(e) = zellij::save_web_url(&url) {
                            eprintln!("Warning: Failed to save web URL: {}", e);
                        }
                        println!("Web client: {}", url);
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to create web token: {}", e);
                    }
                }
                Some(child)
            }
            Err(e) => {
                eprintln!("Warning: Failed to start web server: {}", e);
                None
            }
        }
    } else {
        None
    };

    // Generate the Zellij layout
    if let Err(e) = zellij::generate_layout() {
        eprintln!("Error generating Zellij layout: {}", e);
        std::process::exit(1);
    }

    // Start Zellij with the gz-claude layout
    // Web server cleanup handled by process exit
    if let Err(e) = zellij::start_zellij() {
        eprintln!("Error starting Zellij: {}", e);
        std::process::exit(1);
    }
}

fn run_top_bar() {
    use crossterm::{
        event::{self, Event, KeyCode, KeyEvent},
        terminal::{disable_raw_mode, enable_raw_mode},
    };
    use std::io::Write;
    use std::time::Duration;

    // Check if running inside Zellij
    if std::env::var("ZELLIJ").is_err() {
        eprintln!(
            "Error: gz-claude top-bar must be run inside Zellij.\n\
             Run 'gz-claude' without arguments to start Zellij with the proper layout."
        );
        std::process::exit(1);
    }

    // Get session name from environment
    let session_name = std::env::var("ZELLIJ_SESSION_NAME").ok();

    // Load URL once at startup, with a few retries
    let mut url: Option<String> = None;
    for _ in 0..10 {
        if let Some(u) = zellij::load_web_url() {
            // Insert session name into URL if available
            if let Some(ref session) = session_name {
                // URL format: https://host:port/?token=xxx
                // Change to: https://host:port/session?token=xxx
                if let Some(pos) = u.find("/?token=") {
                    let (base, token_part) = u.split_at(pos);
                    url = Some(format!("{}/{}?token={}", base, session, &token_part[8..]));
                } else {
                    url = Some(u);
                }
            } else {
                url = Some(u);
            }
            break;
        }
        std::thread::sleep(Duration::from_millis(500));
    }

    // Save the updated URL with session name
    if let Some(ref u) = url {
        let _ = zellij::save_web_url(u);
    }

    // Enable raw mode for keyboard input
    let _ = enable_raw_mode();

    let mut copied_message_until: Option<std::time::Instant> = None;
    let mut needs_redraw = true;

    loop {
        if needs_redraw {
            // Clear screen and move to beginning
            print!("\x1B[2J\x1B[H");

            // Check if we should show "Copied!" message
            let show_copied = copied_message_until
                .map(|t| std::time::Instant::now() < t)
                .unwrap_or(false);

            if let Some(ref u) = url {
                if show_copied {
                    print!(" 🌐 {}  ✅ Copied! ", u);
                } else {
                    print!(" 🌐 {}  [c] copy ", u);
                }
            } else {
                print!(" gz-claude ");
            }

            // Flush output
            let _ = std::io::stdout().flush();
            needs_redraw = false;
        }

        // Check if copied message should expire
        if let Some(t) = copied_message_until {
            if std::time::Instant::now() >= t {
                copied_message_until = None;
                needs_redraw = true;
            }
        }

        // Poll for keyboard events (non-blocking with 200ms timeout)
        if event::poll(Duration::from_millis(200)).unwrap_or(false) {
            if let Ok(Event::Key(KeyEvent { code, .. })) = event::read() {
                match code {
                    KeyCode::Char('c') => {
                        // Copy URL to clipboard
                        if let Some(ref u) = url {
                            if zellij::copy_to_clipboard(u).is_ok() {
                                copied_message_until =
                                    Some(std::time::Instant::now() + Duration::from_secs(2));
                                needs_redraw = true;
                            }
                        }
                    }
                    KeyCode::Char('q') => {
                        let _ = disable_raw_mode();
                        break;
                    }
                    _ => {}
                }
            }
        }
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
