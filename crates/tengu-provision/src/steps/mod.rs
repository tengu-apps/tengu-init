//! Installation step definitions
//!
//! Each step implements the [`Step`] trait and can render to both
//! cloud-init YAML fragments and idempotent bash commands.

mod command;
mod directory;
mod file;
mod firewall;
mod package;
mod service;
mod user;

pub use command::RunCommand;
pub use directory::EnsureDirectory;
pub use file::WriteFile;
pub use firewall::{EnsureFirewall, UfwRule};
pub use package::{InstallDebFromUrl, InstallPackage, Repository};
pub use service::EnsureService;
pub use user::EnsureUser;

use serde::Serialize;

/// Result of running a step
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepResult {
    /// Step executed successfully
    Applied,
    /// Step was already satisfied, skipped
    Skipped,
    /// Step failed
    Failed(String),
}

/// A single installation step
///
/// All steps must be:
/// - **Idempotent**: Safe to run multiple times
/// - **Describable**: Have a human-readable description
/// - **Renderable**: Can output both cloud-init YAML and bash
pub trait Step: Send + Sync {
    /// Human-readable description of what this step does
    fn description(&self) -> &str;

    /// Render as cloud-init YAML fragment
    fn to_cloud_init(&self) -> CloudInitFragment;

    /// Render as idempotent bash commands
    fn to_bash(&self) -> Vec<String>;

    /// Check command to determine if step is already satisfied.
    ///
    /// If `Some(cmd)` is returned and the command succeeds (exit 0),
    /// the step will be skipped. If `None`, the step always runs.
    fn check_command(&self) -> Option<String>;
}

/// Fragment that can be merged into a cloud-init config
#[derive(Debug, Default, Clone, Serialize)]
pub struct CloudInitFragment {
    /// Packages to install via apt
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub packages: Vec<String>,

    /// Files to write
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub write_files: Vec<CloudInitFile>,

    /// Commands to run
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub runcmd: Vec<String>,
}

/// A file to write in cloud-init format
#[derive(Debug, Clone, Serialize)]
pub struct CloudInitFile {
    pub path: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
}
