//! Web client support for gz-claude.
//!
//! Provides web server startup functionality for accessing Zellij sessions
//! via the web client.
//!
//! @author waabox(waabox[at]gmail[dot]com)

use std::fs;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

use crate::config::Config;
use crate::error::{GzClaudeError, Result};

/// Get the local IP address of the machine.
///
/// Tries to get the primary network interface IP address.
/// Falls back to "localhost" if unable to determine.
pub fn get_local_ip() -> String {
    // Try to get IP using route command (works on macOS)
    if let Ok(output) = Command::new("sh")
        .args(["-c", "ipconfig getifaddr en0 2>/dev/null || ipconfig getifaddr en1 2>/dev/null"])
        .output()
    {
        let ip = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !ip.is_empty() && ip.contains('.') {
            return ip;
        }
    }

    "localhost".to_string()
}

/// Copy text to the system clipboard (macOS).
pub fn copy_to_clipboard(text: &str) -> Result<()> {
    let mut child = Command::new("pbcopy")
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| GzClaudeError::Zellij(format!("Failed to run pbcopy: {}", e)))?;

    if let Some(stdin) = child.stdin.as_mut() {
        use std::io::Write;
        stdin.write_all(text.as_bytes())
            .map_err(|e| GzClaudeError::Zellij(format!("Failed to write to pbcopy: {}", e)))?;
    }

    child.wait()
        .map_err(|e| GzClaudeError::Zellij(format!("Failed to wait for pbcopy: {}", e)))?;

    Ok(())
}

/// Returns the path to the web URL file.
///
/// The file is stored at `~/.gz-claude/web_url`.
pub fn web_url_path() -> PathBuf {
    Config::default_dir().join("web_url")
}

/// Save the web URL to a file for the top bar to display.
///
/// # Arguments
///
/// * `url` - The complete web URL with token
///
/// # Errors
///
/// Returns an error if the file cannot be written.
pub fn save_web_url(url: &str) -> Result<()> {
    let path = web_url_path();
    fs::write(&path, url)?;
    Ok(())
}

/// Load the saved web URL.
///
/// # Returns
///
/// The saved web URL, or None if not found.
pub fn load_web_url() -> Option<String> {
    let path = web_url_path();
    fs::read_to_string(&path).ok().map(|s| s.trim().to_string())
}

/// Clear the saved web URL.
pub fn clear_web_url() -> Result<()> {
    let path = web_url_path();
    if path.exists() {
        fs::remove_file(&path)?;
    }
    Ok(())
}

/// Create a web token for Zellij web authentication.
///
/// Runs `zellij web --create-token` and parses the output to get the token.
///
/// # Returns
///
/// The token string for web authentication.
///
/// # Errors
///
/// - `GzClaudeError::Zellij` if token creation fails
pub fn create_web_token() -> Result<String> {
    let output = Command::new("zellij")
        .args(["web", "--create-token"])
        .output()
        .map_err(|e| GzClaudeError::Zellij(format!("Failed to create web token: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(GzClaudeError::Zellij(format!("Failed to create web token: {}", stderr)).into());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Output format: "Created token successfully\ntoken_N: <uuid>"
    for line in stdout.lines() {
        // Look for lines like "token_1: uuid-here"
        if line.contains(": ") && line.starts_with("token_") {
            if let Some(token) = line.split(": ").nth(1) {
                return Ok(token.trim().to_string());
            }
        }
    }

    Err(GzClaudeError::Zellij(format!("Could not parse token from output: {}", stdout)).into())
}

/// Returns the path to the SSL directory.
pub fn ssl_dir() -> PathBuf {
    Config::default_dir().join("ssl")
}

/// Returns the path to the SSL certificate file.
pub fn ssl_cert_path() -> PathBuf {
    ssl_dir().join("cert.pem")
}

/// Returns the path to the SSL key file.
pub fn ssl_key_path() -> PathBuf {
    ssl_dir().join("key.pem")
}

/// Check if SSL certificates exist.
pub fn ssl_certs_exist() -> bool {
    ssl_cert_path().exists() && ssl_key_path().exists()
}

/// Generate self-signed SSL certificates for the web server.
///
/// Creates certificates in ~/.gz-claude/ssl/ directory.
/// These allow accessing the web interface from other devices on the network.
///
/// # Returns
///
/// Ok(()) if certificates were created successfully.
///
/// # Errors
///
/// Returns an error if openssl command fails.
pub fn generate_ssl_certs() -> Result<()> {
    let ssl_dir = ssl_dir();
    fs::create_dir_all(&ssl_dir)?;

    let cert_path = ssl_cert_path();
    let key_path = ssl_key_path();

    // Generate self-signed certificate using openssl
    let output = Command::new("openssl")
        .args([
            "req", "-x509", "-newkey", "rsa:2048",
            "-keyout", &key_path.to_string_lossy(),
            "-out", &cert_path.to_string_lossy(),
            "-days", "365",
            "-nodes",
            "-subj", "/CN=gz-claude",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|e| GzClaudeError::Zellij(format!("Failed to run openssl: {}", e)))?;

    if !output.success() {
        return Err(GzClaudeError::Zellij("Failed to generate SSL certificates".to_string()).into());
    }

    Ok(())
}

/// Ensure SSL certificates exist, generating them if needed.
pub fn ensure_ssl_certs() -> Result<()> {
    if !ssl_certs_exist() {
        generate_ssl_certs()?;
    }
    Ok(())
}

/// Start the Zellij web server as a background process.
///
/// If SSL certificates exist in ~/.gz-claude/ssl/, binds to 0.0.0.0 for network access.
/// Otherwise, binds to 127.0.0.1 for local-only access.
///
/// # Arguments
///
/// * `port` - The port number for the web server
///
/// # Returns
///
/// A tuple of (Child process handle, bool indicating if using SSL/network access).
///
/// # Errors
///
/// - `GzClaudeError::Zellij` if the web server fails to start
pub fn start_web_server(_bind_address: &str, port: u16) -> Result<(Child, bool)> {
    let port_str = port.to_string();

    let (child, use_ssl) = if ssl_certs_exist() {
        // Use SSL and bind to all interfaces for network access
        let child = Command::new("zellij")
            .args([
                "web", "--start", "--daemonize",
                "--ip", "0.0.0.0",
                "--port", &port_str,
                "--cert", &ssl_cert_path().to_string_lossy(),
                "--key", &ssl_key_path().to_string_lossy(),
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| GzClaudeError::Zellij(format!("Failed to start web server: {}", e)))?;
        (child, true)
    } else {
        // No SSL, localhost only
        let child = Command::new("zellij")
            .args(["web", "--start", "--daemonize", "--ip", "127.0.0.1", "--port", &port_str])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| GzClaudeError::Zellij(format!("Failed to start web server: {}", e)))?;
        (child, false)
    };

    Ok((child, use_ssl))
}

/// Construct the web client URL with token.
///
/// If use_ssl is true, uses https and the local IP for network access.
/// Otherwise, uses http and localhost for local-only access.
///
/// # Arguments
///
/// * `port` - The port number
/// * `token` - The authentication token
/// * `use_ssl` - Whether SSL is enabled (determines http vs https and host)
///
/// # Returns
///
/// The complete URL for accessing the web client with token.
pub fn web_url(port: u16, token: &str, use_ssl: bool) -> String {
    if use_ssl {
        let ip = get_local_ip();
        format!("https://{}:{}/?token={}", ip, port, token)
    } else {
        format!("http://localhost:{}/?token={}", port, token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn when_getting_local_ip_should_return_valid_ip_or_localhost() {
        let ip = get_local_ip();
        assert!(!ip.is_empty());
        // Should be either localhost or a valid IP
        assert!(ip == "localhost" || ip.contains('.'));
    }

    #[test]
    fn when_getting_web_url_without_ssl_should_use_localhost() {
        let url = web_url(8082, "abc123", false);
        assert!(url.starts_with("http://localhost:"));
        assert!(url.contains(":8082"));
        assert!(url.contains("token=abc123"));
    }

    #[test]
    fn when_getting_web_url_with_ssl_should_use_https_and_local_ip() {
        let url = web_url(8082, "token123", true);
        assert!(url.starts_with("https://"));
        assert!(url.contains(":8082"));
        assert!(url.contains("token=token123"));
    }
}
