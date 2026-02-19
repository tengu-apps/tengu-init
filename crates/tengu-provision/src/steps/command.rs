//! Generic command execution steps

use super::{CloudInitFragment, Step};

/// Run a command with optional idempotency guard
#[derive(Debug, Clone)]
pub struct RunCommand {
    /// Human-readable description
    pub description: String,
    /// Command to execute
    pub command: String,
    /// If this command succeeds (exit 0), skip running `command`
    pub unless: Option<String>,
}

impl RunCommand {
    /// Create a new command step
    pub fn new(description: impl Into<String>, command: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            command: command.into(),
            unless: None,
        }
    }

    /// Add an idempotency guard
    pub fn unless(mut self, check: impl Into<String>) -> Self {
        self.unless = Some(check.into());
        self
    }
}

impl Step for RunCommand {
    fn description(&self) -> &str {
        &self.description
    }

    fn to_cloud_init(&self) -> CloudInitFragment {
        CloudInitFragment {
            runcmd: self.to_bash(),
            ..Default::default()
        }
    }

    fn to_bash(&self) -> Vec<String> {
        if let Some(unless) = &self.unless {
            vec![format!("{} || {{ {}; }}", unless, self.command)]
        } else {
            vec![self.command.clone()]
        }
    }

    fn check_command(&self) -> Option<String> {
        self.unless.clone()
    }
}
