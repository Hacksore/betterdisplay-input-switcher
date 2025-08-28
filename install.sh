#!/bin/bash

set -euo pipefail

# Simple installer for betterdisplay-kvm on macOS.
# - Copies the release binary to /usr/local/libexec/betterdisplay-kvm/betterdisplay-kvm
# - Adds a convenience symlink at /usr/local/bin/betterdisplay-kvm
# - Installs LaunchAgent plist to ~/Library/LaunchAgents
# - (Re)loads the LaunchAgent via launchctl

BIN_NAME="betterdisplay-kvm"
BUILD_BIN="target/release/${BIN_NAME}"
LIBEXEC_DIR="/usr/local/libexec/${BIN_NAME}"
INSTALL_BIN="${LIBEXEC_DIR}/${BIN_NAME}"
USR_LOCAL_BIN="/usr/local/bin/${BIN_NAME}"
PLIST_SRC="pkg/root/Library/LaunchAgents/com.github.hacksore.betterdisplay-kvm.plist"
PLIST_DEST="${HOME}/Library/LaunchAgents/com.github.hacksore.betterdisplay-kvm.plist"

echo "==> Installing ${BIN_NAME}"

if [[ ! -f "${BUILD_BIN}" ]]; then
  echo "Error: ${BUILD_BIN} not found. Build the project first (e.g. cargo build --release)." >&2
  exit 1
fi

echo "==> Creating libexec dir: ${LIBEXEC_DIR}"
sudo mkdir -p "${LIBEXEC_DIR}"

echo "==> Copying binary to ${INSTALL_BIN}"
sudo cp "${BUILD_BIN}" "${INSTALL_BIN}"
sudo chmod 755 "${INSTALL_BIN}"

echo "==> Creating symlink ${USR_LOCAL_BIN} -> ${INSTALL_BIN} (if missing)"
if [[ ! -e "${USR_LOCAL_BIN}" ]]; then
  sudo ln -s "${INSTALL_BIN}" "${USR_LOCAL_BIN}"
fi

echo "==> Installing LaunchAgent plist to ${PLIST_DEST}"
mkdir -p "${HOME}/Library/LaunchAgents"
cp "${PLIST_SRC}" "${PLIST_DEST}"

echo "==> (Re)loading LaunchAgent"
# Try to unload first; ignore errors if not loaded yet
launchctl unload -w "${PLIST_DEST}" >/dev/null 2>&1 || true
launchctl load -w "${PLIST_DEST}"

echo "==> Done. Service label: com.github.hacksore.betterdisplay-kvm"
echo "    Logs: ~/Library/Logs/betterdisplay-kvm.{out,err}.log"
echo "    Binary: ${INSTALL_BIN} (symlink at ${USR_LOCAL_BIN})"

