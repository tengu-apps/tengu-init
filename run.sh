#!/bin/bash
# Test fix: destroy old VM, provision fresh with fixed binary
set -e

echo "=== Destroying old VM ==="
hcloud server delete test-debug --poll-interval 1s 2>/dev/null || true

echo "=== Provisioning fresh VM ==="
~/Projects/tengu-init/target/release/tengu-init --hetzner --direct -y \
    --name test-fix2 --server-type cax41 --location fsn1 \
    --domain-platform test.tengu.to --domain-apps test.tengu.host \
    --notify-email adam.ladachowski@gmail.com \
    --acme-email adam.ladachowski@gmail.com \
    --ssh-key "$(cat ~/.ssh/id_ed25519.pub)" \
    --deb-path /tmp/tengu-debs/tengu_0.2.11-1_arm64.deb \
    --release v0.2.11 \
    -u chi
