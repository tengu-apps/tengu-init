# TODO: E2E Release Test — XFS + Direct TLS

## Pre-Flight

- [x] Build latest tengu .deb (ARM64) — built on runner-arm64
- [x] Build latest tengu .deb (AMD64) — built on runner-amd64
- [x] Build tengu-init binary — v0.5.5
- [x] Verify DNS availability for test subdomains

## Phase 1: ARM64 (cax41)

- [x] Create Hetzner VM (cax41, fsn1, IP 178.105.1.209)
- [x] Run `tengu-init --direct` — XFS steps 30-36 all green
- [x] Verify: `docker info` shows `overlay2` + `xfs`
- [x] Verify: `/var/lib/docker` mounted as XFS with `prjquota`
- [x] Verify: fstab entry for `docker.img`
- [x] Verify: `--storage-opt size=50M` enforced (container sees 50M)
- [x] Verify: `/etc/docker/daemon.json` has `overlay2`
- [x] Verify: all services running (tengu, caddy, docker, postgresql)
- [x] Reboot VM, confirm XFS mount + services survive
- [x] Destroy VM

## Phase 2: AMD64 (cx53)

- [x] Create Hetzner VM (cx53, fsn1, IP 178.105.1.209)
- [x] Run `tengu-init --direct` — XFS steps 30-36 all green
- [x] Verify: same XFS + Direct TLS checks as Phase 1 — all pass
- [x] Reboot VM, confirm persistence — all pass
- [x] Destroy VM

## Phase 3: Cleanup

- [x] Both VMs destroyed
- [ ] Tag release if warranted
