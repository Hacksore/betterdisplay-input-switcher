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

# building in release mode is recommended
cargo build --release

echo "==> Installing ${BIN_NAME}"

if [[ ! -f "${BUILD_BIN}" ]]; then
  echo "Error: ${BUILD_BIN} not found. Build the project first (e.g. cargo build --release)." >&2
  exit 1
fi

echo "==> Creating libexec dir: ${LIBEXEC_DIR}"
sudo mkdir -p "${LIBEXEC_DIR}"

echo "==> Copying binary to ${INSTALL_BIN}"
sudo cp "${BUILD_BIN}" "${INSTALL_BIN}"
sudo chmod 777 "${INSTALL_BIN}"

echo "==> Creating symlink ${USR_LOCAL_BIN} -> ${INSTALL_BIN} (if missing)"
if [[ ! -e "${USR_LOCAL_BIN}" ]]; then
  sudo ln -s "${INSTALL_BIN}" "${USR_LOCAL_BIN}"
fi

# run the thing with a --install flag that will exit after it configures the launch
./target/release/betterdisplay-kvm --install

echo "done"
