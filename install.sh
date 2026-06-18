#!/usr/bin/env bash

set -euo pipefail

REPO="suy0x1/phantom_relay"

BIN_DIR="/usr/local/bin"
CONFIG_DIR="/etc/phantomrelay"

SERVICE_NAME="phantomrelayd"
SERVICE_FILE="/etc/systemd/system/${SERVICE_NAME}.service"

if [[ $EUID -ne 0 ]]; then
echo "error: run as root"
exit 1
fi

require() {
command -v "$1" >/dev/null 2>&1 || {
echo "error: missing dependency: $1"
exit 1
}
}

require curl
require sha256sum
require install
require systemctl

if [[ "$(uname -m)" != "x86_64" ]]; then
echo "error: only x86_64 GNU builds are currently supported"
exit 1
fi

VERSION="${1:-latest}"

if [[ "$VERSION" == "latest" ]]; then
VERSION="$(curl -fsSL 
"https://api.github.com/repos/${REPO}/releases/latest" 
| grep '"tag_name"' 
| cut -d '"' -f4)"
fi

echo "[+] Installing PhantomRelay ${VERSION}"
echo

TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT

cd "$TMPDIR"

BASE_URL="https://github.com/${REPO}/releases/download/${VERSION}"

DAEMON_ASSET="phantomrelayd-linux-x86_64-gnu"
CLI_ASSET="prctl-linux-x86_64-gnu"

echo "[1/6] Downloading release assets..."

curl -fsSLO "${BASE_URL}/${DAEMON_ASSET}"
curl -fsSLO "${BASE_URL}/${CLI_ASSET}"
curl -fsSLO "${BASE_URL}/phantomrelay.toml"
curl -fsSLO "${BASE_URL}/SHA256SUMS"

echo "[2/6] Verifying checksums..."

grep "${DAEMON_ASSET}" SHA256SUMS > verify.txt
grep "${CLI_ASSET}" SHA256SUMS >> verify.txt
grep "phantomrelay.toml" SHA256SUMS >> verify.txt

sha256sum -c verify.txt

rm verify.txt

echo "[3/6] Installing binaries..."

install -Dm755 "${DAEMON_ASSET}" "${BIN_DIR}/phantomrelayd"
install -Dm755 "${CLI_ASSET}" "${BIN_DIR}/prctl"

echo "[4/6] Installing configuration..."

mkdir -p "${CONFIG_DIR}"

if [[ ! -f "${CONFIG_DIR}/phantomrelay.toml" ]]; then
install -Dm644 phantomrelay.toml 
"${CONFIG_DIR}/phantomrelay.toml"
echo "      Config created."
else
echo "      Existing config preserved."
fi

echo "[5/6] Creating systemd service..."

cat > "${SERVICE_FILE}" << 'EOF'
[Unit]
Description=PhantomRelay
After=network-online.target
Wants=network-online.target

[Service]
Type=simple

ExecStart=/usr/local/bin/phantomrelayd

Restart=always
RestartSec=5

User=root
Group=root

LimitNOFILE=1048576
LimitNPROC=1048576
TasksMax=infinity

OOMScoreAdjust=-900

StandardOutput=journal
StandardError=journal

NoNewPrivileges=true
PrivateTmp=true
ProtectHome=true

[Install]
WantedBy=multi-user.target
EOF

echo "[6/6] Enabling service..."

systemctl daemon-reload
systemctl enable phantomrelayd
systemctl restart phantomrelayd

echo
echo "========================================="
echo " PhantomRelay installed successfully"
echo "========================================="
echo
echo "Version: ${VERSION}"
echo
echo "Config:"
echo "  /etc/phantomrelay/phantomrelay.toml"
echo
echo "Service:"
echo "  systemctl status phantomrelayd"
echo
echo "Logs:"
echo "  journalctl -u phantomrelayd -f"
echo
echo "CLI:"
echo "  prctl"
echo
