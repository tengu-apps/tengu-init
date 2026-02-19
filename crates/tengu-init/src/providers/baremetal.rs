//! Baremetal provider - provision via SSH
//!
//! Connects to an existing server via SSH, uploads a bash script,
//! and executes it with real-time progress streaming.

use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::time::Duration;

use anyhow::{Context, Result, bail};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use tengu_provision::{BashRenderer, Manifest, Renderer, TenguConfig};

/// Baremetal server provisioning via SSH
pub struct Baremetal {
    /// SSH host (can be user@host format)
    pub host: String,
    /// SSH user (optional, extracted from host or defaults to "root")
    pub user: String,
    /// SSH port
    pub port: u16,
}

impl Baremetal {
    /// Create a new baremetal provider from a host specification
    ///
    /// Host can be:
    /// - `hostname` (uses default user "root")
    /// - `user@hostname` (extracts user)
    pub fn new(host: &str, port: u16) -> Self {
        let (user, hostname) = if let Some((u, h)) = host.split_once('@') {
            (u.to_string(), h.to_string())
        } else {
            ("root".to_string(), host.to_string())
        };

        Self {
            host: hostname,
            user,
            port,
        }
    }

    /// Generate the provisioning bash script
    pub fn generate_script(config: &TenguConfig) -> Result<String> {
        let manifest = Manifest::tengu(config);
        let renderer = BashRenderer::new().verbose(true).color(true);
        renderer
            .render(&manifest)
            .map_err(|e| anyhow::anyhow!("Failed to render script: {e:?}"))
    }

    /// Provision the server
    ///
    /// 1. Generate bash script from config
    /// 2. Upload to /tmp/tengu-provision.sh via SSH
    /// 3. Execute with sudo, streaming output
    /// 4. Parse progress markers and display pretty progress
    /// 5. Cleanup temp script
    pub fn provision(&self, config: &TenguConfig) -> Result<()> {
        // Generate script
        println!("\n{} Generating provisioning script...", style("*").cyan());
        let script = Self::generate_script(config)?;

        // Count steps from manifest
        let manifest = Manifest::tengu(config);
        let total_steps = manifest.steps.len();

        // Wait for SSH
        self.wait_for_ssh()?;

        // Upload script
        println!(
            "{} Uploading script to {}...",
            style("*").cyan(),
            self.ssh_destination()
        );
        self.upload_script(&script)?;

        // Execute script
        println!("{} Executing provisioning script...\n", style("*").cyan());
        println!("{}", style("-".repeat(50)).dim());
        self.execute_script(total_steps)?;
        println!("{}", style("-".repeat(50)).dim());

        // Cleanup
        println!("{} Cleaning up...", style("*").cyan());
        self.cleanup_script()?;

        Ok(())
    }

    /// SSH destination string (user@host)
    fn ssh_destination(&self) -> String {
        format!("{}@{}", self.user, self.host)
    }

    /// SSH command arguments (common options)
    fn ssh_args(&self) -> Vec<String> {
        vec![
            "-o".into(),
            "StrictHostKeyChecking=no".into(),
            "-o".into(),
            "UserKnownHostsFile=/dev/null".into(),
            "-o".into(),
            "LogLevel=ERROR".into(),
            "-p".into(),
            self.port.to_string(),
        ]
    }

    /// Wait for SSH to become available
    fn wait_for_ssh(&self) -> Result<()> {
        let spinner = ProgressBar::new_spinner();
        spinner.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.cyan} {msg}")
                .unwrap(),
        );
        spinner.set_message(format!("Connecting to {}:{}...", self.host, self.port));
        spinner.enable_steady_tick(Duration::from_millis(100));

        let mut attempts = 0;
        let max_attempts = 30;

        loop {
            let mut args = self.ssh_args();
            args.extend([
                "-o".into(),
                "ConnectTimeout=5".into(),
                "-o".into(),
                "BatchMode=yes".into(),
                self.ssh_destination(),
                "true".into(),
            ]);

            let status = Command::new("ssh")
                .args(&args)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();

            if status.map(|s| s.success()).unwrap_or(false) {
                break;
            }

            attempts += 1;
            if attempts >= max_attempts {
                spinner.finish_with_message(format!(
                    "{} Failed to connect after {} attempts",
                    style("x").red(),
                    max_attempts
                ));
                bail!("Could not connect to {}:{} via SSH", self.host, self.port);
            }

            std::thread::sleep(Duration::from_secs(2));
        }

        spinner.finish_with_message(format!("{} SSH connection established", style("v").green()));
        Ok(())
    }

    /// Upload script to remote server
    fn upload_script(&self, script: &str) -> Result<()> {
        let mut args = self.ssh_args();
        args.push(self.ssh_destination());
        args.push("cat > /tmp/tengu-provision.sh && chmod +x /tmp/tengu-provision.sh".into());

        let mut child = Command::new("ssh")
            .args(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to start SSH for upload")?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(script.as_bytes())
                .context("Failed to write script to SSH")?;
        }

        let output = child
            .wait_with_output()
            .context("Failed to upload script")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("Failed to upload script: {stderr}");
        }

        Ok(())
    }

    /// Execute script and stream progress
    fn execute_script(&self, total_steps: usize) -> Result<()> {
        let mut args = self.ssh_args();
        args.push("-t".into()); // Allocate PTY for better output
        args.push(self.ssh_destination());
        args.push("sudo /tmp/tengu-provision.sh".into());

        let mut child = Command::new("ssh")
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to execute script")?;

        let stdout = child.stdout.take().context("No stdout")?;
        let reader = BufReader::new(stdout);

        // Track current step for spinner
        let mut current_spinner: Option<ProgressBar> = None;

        for line in reader.lines() {
            let Ok(line) = line else { continue };

            // Parse progress markers
            if let Some(marker) = parse_progress_marker(&line) {
                match marker {
                    ProgressMarker::Start { step, desc } => {
                        // Finish previous spinner if any
                        if let Some(spinner) = current_spinner.take() {
                            spinner.finish_and_clear();
                        }

                        // Start new spinner
                        let spinner = ProgressBar::new_spinner();
                        spinner.set_style(
                            ProgressStyle::default_spinner()
                                .template(&format!(
                                    "{{spinner:.cyan}} [{step}/{total_steps}] {{msg}}"
                                ))
                                .unwrap(),
                        );
                        spinner.set_message(desc);
                        spinner.enable_steady_tick(Duration::from_millis(100));
                        current_spinner = Some(spinner);
                    }
                    ProgressMarker::Done { step, desc } => {
                        if let Some(spinner) = current_spinner.take() {
                            spinner.finish_and_clear();
                        }
                        println!("[{}/{}] {} {}", step, total_steps, style("v").green(), desc);
                    }
                    ProgressMarker::Skip { step, desc } => {
                        if let Some(spinner) = current_spinner.take() {
                            spinner.finish_and_clear();
                        }
                        println!(
                            "[{}/{}] {} {} {}",
                            step,
                            total_steps,
                            style("o").yellow(),
                            desc,
                            style("(skipped)").dim()
                        );
                    }
                    ProgressMarker::Fail { step, desc } => {
                        if let Some(spinner) = current_spinner.take() {
                            spinner.finish_and_clear();
                        }
                        println!("[{}/{}] {} {}", step, total_steps, style("x").red(), desc);
                    }
                    ProgressMarker::Complete { .. } => {
                        if let Some(spinner) = current_spinner.take() {
                            spinner.finish_and_clear();
                        }
                    }
                }
            }
            // Skip non-marker lines (we show progress via markers)
        }

        // Clean up any remaining spinner
        if let Some(spinner) = current_spinner.take() {
            spinner.finish_and_clear();
        }

        let status = child.wait().context("Failed to wait for script")?;

        if !status.success() {
            bail!("Provisioning script failed with exit code: {status}");
        }

        Ok(())
    }

    /// Remove the temporary script
    fn cleanup_script(&self) -> Result<()> {
        let mut args = self.ssh_args();
        args.push(self.ssh_destination());
        args.push("rm -f /tmp/tengu-provision.sh".into());

        let status = Command::new("ssh")
            .args(&args)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .context("Failed to cleanup script")?;

        if !status.success() {
            // Non-fatal, just warn
            eprintln!(
                "{} Warning: Could not remove temp script",
                style("!").yellow()
            );
        }

        Ok(())
    }
}

/// Progress marker types
enum ProgressMarker {
    Start { step: usize, desc: String },
    Done { step: usize, desc: String },
    Skip { step: usize, desc: String },
    Fail { step: usize, desc: String },
    Complete { _total: usize },
}

/// Parse a progress marker from a line
///
/// Format: `TENGU_STEP:ACTION:step_num:description`
fn parse_progress_marker(line: &str) -> Option<ProgressMarker> {
    let line = line.trim();

    // Strip ANSI escape codes for parsing
    let clean = strip_ansi_codes(line);

    if !clean.starts_with("TENGU_STEP:") {
        return None;
    }

    let parts: Vec<&str> = clean.splitn(4, ':').collect();
    if parts.len() < 3 {
        return None;
    }

    let action = parts[1];
    let step: usize = parts[2].parse().ok()?;
    let desc = parts.get(3).unwrap_or(&"").to_string();

    match action {
        "START" => Some(ProgressMarker::Start { step, desc }),
        "DONE" => Some(ProgressMarker::Done { step, desc }),
        "SKIP" => Some(ProgressMarker::Skip { step, desc }),
        "FAIL" => Some(ProgressMarker::Fail { step, desc }),
        "COMPLETE" => Some(ProgressMarker::Complete { _total: step }),
        _ => None,
    }
}

/// Strip ANSI escape codes from a string
fn strip_ansi_codes(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // Skip escape sequence
            if chars.peek() == Some(&'[') {
                chars.next(); // consume '['
                // Skip until we hit a letter (end of sequence)
                while let Some(&next) = chars.peek() {
                    chars.next();
                    if next.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
        } else {
            result.push(c);
        }
    }

    result
}
