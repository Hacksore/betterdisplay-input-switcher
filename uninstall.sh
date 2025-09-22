#!/bin/bash

set -euo pipefail

BIN_NAME="betterdisplay-kvm"
LIBEXEC_DIR="/usr/local/libexec/${BIN_NAME}"
INSTALL_BIN="${LIBEXEC_DIR}/${BIN_NAME}"
USR_LOCAL_BIN="/usr/local/bin/${BIN_NAME}"
PLIST_DEST="${HOME}/Library/LaunchAgents/com.github.hacksore.betterdisplay-kvm.plist"

echo "==> Stopping ${BIN_NAME} service if running"
if launchctl list | grep -q "com.github.hacksore.betterdisplay-kvm"; then
  echo "==> Service is running, unloading..."
  launchctl unload "${PLIST_DEST}" || true
fi

echo "==> Uninstalling ${BIN_NAME}"

launchctl unload "${PLIST_DEST}"

sudo rm -rf "${USR_LOCAL_BIN}" "${INSTALL_BIN}" "${PLIST_DEST}"
