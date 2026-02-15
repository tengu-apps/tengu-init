//! Hetzner Cloud provider
//!
//! Currently uses the `hcloud` CLI. Requires:
//! ```sh
//! brew install hcloud
//! hcloud context create tengu
//! ```

use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;
use std::thread;

use anyhow::{bail, Context, Result};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};

/// Server creation parameters
pub struct ServerParams<'a> {
    pub name: &'a str,
    pub server_type: &'a str,
    pub image: &'a str,
    pub location: &'a str,
    pub cloud_init_path: &'a Path,
}

/// Hetzner Cloud provider (via hcloud CLI)
pub struct Hetzner;

impl Hetzner {
    /// Get server type info (cores, RAM, architecture)
    pub fn server_type_info(server_type: &str) -> Result<String> {
        let output = Command::new("hcloud")
            .args([
                "server-type",
                "describe",
                server_type,
                "-o",
                "format={{.Cores}} cores, {{.Memory}}GB RAM, {{.Architecture}}",
            ])
            .output()
            .context("Failed to run hcloud - is it installed?")?;

        if !output.status.success() {
            bail!("Unknown server type: {}", server_type);
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Check if a server with the given name exists
    pub fn server_exists(name: &str) -> Result<bool> {
        let status = Command::new("hcloud")
            .args(["server", "describe", name])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .context("Failed to run hcloud")?;

        Ok(status.success())
    }

    /// Delete a server by name
    pub fn delete_server(name: &str) -> Result<()> {
        let spinner = ProgressBar::new_spinner();
        spinner.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.cyan} {msg}")
                .unwrap(),
        );
        spinner.set_message(format!("Deleting {}...", name));
        spinner.enable_steady_tick(Duration::from_millis(100));

        let status = Command::new("hcloud")
            .args(["server", "delete", name])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .context("Failed to delete server")?;

        if !status.success() {
            spinner.finish_with_message(format!("{} Failed to delete server", style("✗").red()));
            bail!("Failed to delete server");
        }

        spinner.finish_with_message(format!("{} Deleted {}", style("✓").green(), name));
        thread::sleep(Duration::from_secs(2));
        Ok(())
    }

    /// Create a new server, returns the IP address
    pub fn create_server(params: &ServerParams) -> Result<String> {
        let spinner = ProgressBar::new_spinner();
        spinner.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.cyan} {msg}")
                .unwrap(),
        );
        spinner.set_message(format!("Provisioning {} on Hetzner...", params.name));
        spinner.enable_steady_tick(Duration::from_millis(100));

        let output = Command::new("hcloud")
            .args([
                "server",
                "create",
                "--name",
                params.name,
                "--type",
                params.server_type,
                "--image",
                params.image,
                "--location",
                params.location,
                "--user-data-from-file",
                params.cloud_init_path.to_str().unwrap(),
            ])
            .output()
            .context("Failed to create server")?;

        if !output.status.success() {
            spinner.finish_with_message(format!("{} Failed to create server", style("✗").red()));
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("Failed to create server: {}", stderr);
        }

        spinner.finish_with_message(format!("{} Server created", style("✓").green()));

        // Get IP
        let output = Command::new("hcloud")
            .args(["server", "ip", params.name])
            .output()
            .context("Failed to get server IP")?;

        let ip = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(ip)
    }

    /// Remove old SSH host key for an IP
    pub fn clear_host_key(ip: &str) {
        let _ = Command::new("ssh-keygen")
            .args(["-R", ip])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
}
