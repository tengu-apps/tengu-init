//! Directory management steps

use super::{CloudInitFragment, Step};

/// Ensure a directory exists
#[derive(Debug, Clone)]
pub struct EnsureDirectory {
    /// Directory path
    pub path: String,
    /// Directory permissions (e.g., "0755")
    pub permissions: Option<String>,
    /// Directory owner (e.g., "root:root")
    pub owner: Option<String>,
    /// Description
    description: String,
}

impl EnsureDirectory {
    /// Create a new directory step
    pub fn new(path: impl Into<String>) -> Self {
        let path = path.into();
        let description = format!("Ensure directory {path}");
        Self {
            path,
            permissions: None,
            owner: None,
            description,
        }
    }

    /// Set directory permissions
    pub fn with_permissions(mut self, perms: impl Into<String>) -> Self {
        self.permissions = Some(perms.into());
        self
    }

    /// Set directory owner
    pub fn with_owner(mut self, owner: impl Into<String>) -> Self {
        self.owner = Some(owner.into());
        self
    }
}

impl Step for EnsureDirectory {
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
        let mut cmds = vec![format!("mkdir -p {}", self.path)];

        if let Some(perms) = &self.permissions {
            cmds.push(format!("chmod {} {}", perms, self.path));
        }

        if let Some(owner) = &self.owner {
            cmds.push(format!("chown {} {}", owner, self.path));
        }

        cmds
    }

    fn check_command(&self) -> Option<String> {
        Some(format!("[ -d {} ]", self.path))
    }
}
