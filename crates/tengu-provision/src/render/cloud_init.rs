//! Cloud-init YAML renderer

use crate::{Manifest, TenguConfig};

use super::Renderer;

/// Renders a manifest as cloud-init YAML
#[derive(Debug, Clone, Default)]
pub struct CloudInitRenderer {
    /// Optional user configuration for cloud-init users section
    user_config: Option<CloudInitUserConfig>,
}

/// User configuration for cloud-init
#[derive(Debug, Clone)]
struct CloudInitUserConfig {
    name: String,
    groups: Vec<String>,
    shell: String,
    sudo: String,
    ssh_authorized_keys: Vec<String>,
}

impl CloudInitRenderer {
    /// Create a new cloud-init renderer
    pub fn new() -> Self {
        Self { user_config: None }
    }

    /// Render with configuration context (includes user setup in cloud-init native format)
    pub fn render_with_config(
        &self,
        manifest: &Manifest,
        config: &TenguConfig,
    ) -> Result<String, serde_yaml::Error> {
        // Create a renderer with user config extracted from TenguConfig
        let renderer = Self {
            user_config: Some(CloudInitUserConfig {
                name: config.user.clone(),
                groups: vec!["sudo".into(), "docker".into()],
                shell: "/bin/bash".into(),
                sudo: "ALL=(ALL) NOPASSWD:ALL".into(),
                ssh_authorized_keys: config.ssh_keys.clone(),
            }),
        };
        renderer.render(manifest)
    }
}

impl Renderer for CloudInitRenderer {
    type Output = String;
    type Error = serde_yaml::Error;

    fn render(&self, manifest: &Manifest) -> Result<String, Self::Error> {
        use serde::Serialize;

        #[derive(Serialize)]
        struct CloudInitUser {
            name: String,
            groups: String,
            shell: String,
            sudo: String,
            #[serde(skip_serializing_if = "Vec::is_empty")]
            ssh_authorized_keys: Vec<String>,
        }

        #[derive(Serialize)]
        struct CloudInitConfig {
            hostname: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            fqdn: Option<String>,
            timezone: String,
            locale: String,
            ssh_pwauth: bool,
            disable_root: bool,
            #[serde(skip_serializing_if = "Vec::is_empty")]
            users: Vec<CloudInitUser>,
            package_update: bool,
            package_upgrade: bool,
            #[serde(skip_serializing_if = "Vec::is_empty")]
            packages: Vec<String>,
            #[serde(skip_serializing_if = "Vec::is_empty")]
            write_files: Vec<serde_yaml::Value>,
            #[serde(skip_serializing_if = "Vec::is_empty")]
            runcmd: Vec<String>,
            final_message: String,
        }

        let mut packages = vec![];
        let mut write_files = vec![];
        let mut runcmd = vec![];

        for step in &manifest.steps {
            let fragment = step.to_cloud_init();
            packages.extend(fragment.packages);
            for file in fragment.write_files {
                write_files.push(serde_yaml::to_value(&file)?);
            }
            runcmd.extend(fragment.runcmd);
        }

        // Deduplicate packages
        packages.sort();
        packages.dedup();

        // Build users list
        let users = if let Some(user_cfg) = &self.user_config {
            vec![CloudInitUser {
                name: user_cfg.name.clone(),
                groups: user_cfg.groups.join(", "),
                shell: user_cfg.shell.clone(),
                sudo: user_cfg.sudo.clone(),
                ssh_authorized_keys: user_cfg.ssh_authorized_keys.clone(),
            }]
        } else {
            vec![]
        };

        let config = CloudInitConfig {
            hostname: manifest.hostname.clone(),
            fqdn: manifest.fqdn.clone(),
            timezone: manifest.timezone.clone(),
            locale: manifest.locale.clone(),
            ssh_pwauth: false,
            disable_root: true,
            users,
            package_update: true,
            package_upgrade: true,
            packages,
            write_files,
            runcmd,
            final_message: "Tengu PaaS server ready!".into(),
        };

        let yaml = serde_yaml::to_string(&config)?;
        Ok(format!("#cloud-config\n{yaml}"))
    }
}
