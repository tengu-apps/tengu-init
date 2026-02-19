//! Firewall (UFW) management steps

use super::{CloudInitFragment, Step};

/// A UFW allow rule
#[derive(Debug, Clone)]
pub struct UfwRule {
    /// Port/protocol to allow (e.g., "22/tcp", "80/tcp")
    pub allow: String,
}

impl UfwRule {
    /// Create a new UFW rule
    pub fn new(allow: impl Into<String>) -> Self {
        Self {
            allow: allow.into(),
        }
    }
}

/// Ensure UFW firewall is configured and enabled
#[derive(Debug, Clone)]
pub struct EnsureFirewall {
    /// Rules to apply
    pub rules: Vec<UfwRule>,
    /// Default incoming policy
    pub default_incoming: String,
    /// Default outgoing policy
    pub default_outgoing: String,
    /// Description
    description: String,
}

impl EnsureFirewall {
    /// Create a new firewall step with deny incoming / allow outgoing defaults
    pub fn new() -> Self {
        Self {
            rules: vec![],
            default_incoming: "deny".into(),
            default_outgoing: "allow".into(),
            description: "Configure firewall".into(),
        }
    }

    /// Add a rule to allow a port
    pub fn allow(mut self, port: impl Into<String>) -> Self {
        self.rules.push(UfwRule::new(port));
        self
    }

    /// Set default incoming policy
    pub fn default_incoming(mut self, policy: impl Into<String>) -> Self {
        self.default_incoming = policy.into();
        self
    }

    /// Set default outgoing policy
    pub fn default_outgoing(mut self, policy: impl Into<String>) -> Self {
        self.default_outgoing = policy.into();
        self
    }
}

impl Default for EnsureFirewall {
    fn default() -> Self {
        Self::new()
    }
}

impl Step for EnsureFirewall {
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
        let mut cmds = vec![
            format!("ufw default {} incoming", self.default_incoming),
            format!("ufw default {} outgoing", self.default_outgoing),
        ];

        for rule in &self.rules {
            // ufw allow is already idempotent
            cmds.push(format!("ufw allow {}", rule.allow));
        }

        // Enable if not already
        cmds.push("ufw status | grep -q 'Status: active' || ufw --force enable".to_string());

        cmds
    }

    fn check_command(&self) -> Option<String> {
        Some("ufw status | grep -q 'Status: active'".to_string())
    }
}
