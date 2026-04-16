# E2E Release Test — XFS Docker Backing + Direct TLS Validation

Two features need E2E validation on fresh Hetzner VMs:
1. **Phase 16 "Byarlant"** (tengu) — XFS loopback backing for Docker overlay2 quotas
2. **Phase 5.8 "Atlas"** (tengu-init) — Direct TLS mode (`--direct` flag)

Both are code-complete and committed. This is ops-only — no code changes expected.

## Pre-Flight

Before spinning up VMs:
1. Build latest tengu .deb (ARM64 + AMD64) via CI or locally
2. Ensure tengu-init binary is current (`cargo build --release`)
3. Verify DNS: `test-arm.tengu.host` and `test-amd.tengu.host` available for A records

## Phase 1: ARM64 Test (cax21, FSN1)

### 1.1 Provision
```bash
hcloud server create --name test-arm --type cax41 --location fsn1 --image ubuntu-24.04
# Note the IP, create DNS A records:
#   test-arm.tengu.host → <IP>
#   *.test-arm.tengu.host → <IP>  (if testing app deploy)

tengu-init --direct --hetzner -y \
    --server-type cax41 --location fsn1 \
    --deb-path <path-to-arm64-deb>
```

Must complete in ONE clean run with zero manual fixes.

### 1.2 Verify XFS Docker Backing
```bash
ssh root@<IP> "docker info | grep -E 'Storage Driver|Backing Filesystem'"
# Expected: overlay2 / xfs

ssh root@<IP> "mount | grep /var/lib/docker"
# Expected: xfs with prjquota

ssh root@<IP> "grep docker.img /etc/fstab"
# Expected: fstab entry present

ssh root@<IP> "docker run --rm --storage-opt size=50M alpine df -h /"
# Expected: 50M filesystem (NOT full disk)

ssh root@<IP> "cat /etc/docker/daemon.json"
# Expected: {"storage-driver": "overlay2"}
```

### 1.3 Verify Direct TLS
```bash
ssh root@<IP> "systemctl status caddy tengu docker postgresql"
# All active

ssh root@<IP> "grep -c cloudflare /etc/caddy/Caddyfile"
# Expected: 0 (no CF references)

ssh root@<IP> "ls /etc/systemd/system/caddy.service.d/ 2>/dev/null"
# Expected: empty or no such directory
```

### 1.4 Deploy Test App
```bash
tengu create test-app
# git push a minimal static app
curl -sf https://test-app.test-arm.tengu.host/
# Expected: 200
```

### 1.5 Reboot Persistence
```bash
ssh root@<IP> "reboot"
# Wait 30s
ssh root@<IP> "docker info | grep Backing && docker ps && mount | grep docker"
# Everything back up, XFS mounted, containers running
```

### 1.6 Destroy
```bash
hcloud server delete test-arm
# Remove DNS records
```

## Phase 2: AMD64 Test (cx22, FSN1)

Identical procedure as Phase 1, using:
- `--server-type cx42` (16 vCPU, 32GB — AMD64 equivalent of production cax41)
- AMD64 .deb package

## Phase 3: Cleanup

- Remove all test DNS records
- If both pass: update tengu-init `DEFAULT_RELEASE` to current tag
- Tag release if warranted

## Failure Protocol

If provisioning fails at any point:
1. Note the exact step and error
2. Fix in source
3. **Destroy** the VM completely (not fix in place)
4. Create a new fresh VM
5. Re-run from scratch
6. Repeat until clean

## Cost

| VM | Type | Cost/hr | Est. time | Total |
|----|------|---------|-----------|-------|
| test-arm | cax41 | €0.054 | ~30 min | ~€0.03 |
| test-amd | cx42 | €0.054 | ~30 min | ~€0.03 |
| **Total** | | | | **~€0.06** |
