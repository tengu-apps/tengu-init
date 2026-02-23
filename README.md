<p align="center">
  <a href="https://tengu.to">
    <img src="logo.png" alt="Tengu" width="120" height="120">
  </a>
</p>

<h1 align="center">tengu-init</h1>

<p align="center">
  <strong>Provision Tengu PaaS servers</strong>
</p>

<p align="center">
  One command to spin up a fully configured <a href="https://tengu.to">Tengu</a> server with SSL, PostgreSQL, Docker, and git push deploys.
</p>

---

## Features

- **Hetzner Cloud** - Automatic VM provisioning via cloud-init
- **Baremetal** - SSH-based provisioning for existing servers
- **Idempotent** - Safe to re-run, only applies needed changes
- **Multi-arch** - Supports both ARM64 and x86_64

## Install

```bash
cargo install tengu-init
```

**Requires:** [hcloud CLI](https://github.com/hetznercloud/cli) configured with an API token (for Hetzner provisioning).

## Usage

### Hetzner Cloud (default)

```bash
# Interactive provisioning
tengu-init

# Preview without creating
tengu-init --dry-run

# Override server type
tengu-init --server-type cpx41
```

### Baremetal Server

```bash
# Provision existing server via SSH
tengu-init baremetal chi@192.168.1.100

# Custom SSH port
tengu-init baremetal chi@my-server.com --port 2222

# Generate script only (don't execute)
tengu-init baremetal chi@server --script-only > provision.sh
```

### Show Generated Config

```bash
# View cloud-init YAML
tengu-init show cloud-init

# View bash script
tengu-init show bash
```

## Configuration

Create `~/.config/tengu/init.toml`:

```toml
[server]
name = "tengu"
type = "cax41"        # ARM64, 16 vCPU, 32GB RAM
location = "hel1"     # Helsinki
image = "ubuntu-24.04"

[domains]
platform = "tengu.to"
apps = "tengu.host"

[cloudflare]
api_key = "your-cloudflare-api-key"
email = "your-email@example.com"

[resend]
api_key = "re_xxx"

[ssh]
public_key = "ssh-ed25519 AAAA..."

[notifications]
email = "notify@example.com"
```

## Server Types

| Type | Arch | vCPU | RAM | Disk | Price |
|------|------|------|-----|------|-------|
| `cax11` | ARM64 | 2 | 4GB | 40GB | ~€4/mo |
| `cax21` | ARM64 | 4 | 8GB | 80GB | ~€8/mo |
| `cax31` | ARM64 | 8 | 16GB | 160GB | ~€15/mo |
| `cax41` | ARM64 | 16 | 32GB | 320GB | ~€30/mo |
| `cpx11` | x86 | 2 | 2GB | 40GB | ~€5/mo |
| `cpx21` | x86 | 3 | 4GB | 80GB | ~€9/mo |
| `cpx31` | x86 | 4 | 8GB | 160GB | ~€17/mo |
| `cpx41` | x86 | 8 | 16GB | 240GB | ~€32/mo |

ARM servers (cax*) are recommended—cheaper and Tengu builds for both architectures.

## What Gets Installed

| Component | Description |
|-----------|-------------|
| **Docker** | Container runtime (docker.io) |
| **tengu-caddy** | Caddy with Cloudflare DNS plugin for automatic HTTPS |
| **PostgreSQL 18** | Database with pgvector extension for AI/embeddings |
| **Ollama** | Local LLM inference for RAG features |
| **Tengu** | PaaS server with SSH git endpoint and API |
| **fail2ban** | Intrusion prevention |
| **ufw** | Firewall (ports 22, 80, 443) |

## Project Structure

```
tengu-init/
├── crates/
│   ├── tengu-init/        # CLI binary
│   │   └── templates/     # Tera templates for cloud-init
│   └── tengu-provision/   # Library for provisioning steps
│       └── src/
│           ├── steps/     # Idempotent installation steps
│           ├── render/    # Output renderers (cloud-init, bash)
│           └── manifest.rs
```

## License

MIT
