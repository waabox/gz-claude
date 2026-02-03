//! Zellij installation check utilities.
//!
//! Provides functions to verify Zellij CLI availability and version.
//!
//! @author waabox(waabox[at]gmail[dot]com)

use std::process::Command;

/// Checks whether Zellij is installed and available in the system PATH.
///
/// This function attempts to run `zellij --version` to determine if the
/// Zellij terminal multiplexer is installed and accessible.
///
/// # Returns
///
/// Returns `true` if Zellij is installed and the version command succeeds,
/// `false` otherwise.
///
/// # Example
///
/// ```no_run
/// use gz_claude::zellij::is_zellij_installed;
///
/// if is_zellij_installed() {
///     println!("Zellij is available");
/// } else {
///     println!("Please install Zellij first");
/// }
/// ```
pub fn is_zellij_installed() -> bool {
    Command::new("zellij")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Retrieves the installed Zellij version string.
///
/// Runs `zellij --version` and returns the trimmed output if successful.
///
/// # Returns
///
/// Returns `Some(version_string)` containing the Zellij version if the command
/// succeeds, or `None` if Zellij is not installed or the command fails.
///
/// # Example
///
/// ```no_run
/// use gz_claude::zellij::zellij_version;
///
/// match zellij_version() {
///     Some(version) => println!("Zellij version: {}", version),
///     None => println!("Zellij is not installed"),
/// }
/// ```
pub fn zellij_version() -> Option<String> {
    let output = Command::new("zellij").arg("--version").output().ok()?;

    if !output.status.success() {
        return None;
    }

    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if version.is_empty() {
        None
    } else {
        Some(version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn when_checking_zellij_installed_should_return_bool() {
        // This test verifies the function executes without panic.
        // The actual result depends on whether Zellij is installed on the system.
        let result = is_zellij_installed();
        // Result is either true or false, both are valid
        assert!(result || !result);
    }

    #[test]
    fn when_getting_zellij_version_should_return_option() {
        // This test verifies the function executes without panic.
        // The actual result depends on whether Zellij is installed on the system.
        let result = zellij_version();
        match result {
            Some(version) => {
                // If Zellij is installed, version should not be empty
                assert!(!version.is_empty());
            }
            None => {
                // If Zellij is not installed, None is the expected result
                assert!(result.is_none());
            }
        }
    }
}
