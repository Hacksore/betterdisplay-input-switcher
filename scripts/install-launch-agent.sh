#!/bin/sh
set -euo pipefail

PLIST_SRC_DIR="$(cd "$(dirname "$0")/.." && pwd)/contrib/launch"
PLIST_NAME="com.github.hacksore.betterdisplay-kvm.plist"
PLIST_SRC_PATH="$PLIST_SRC_DIR/$PLIST_NAME"
LAUNCH_AGENTS_DIR="$HOME/Library/LaunchAgents"
PLIST_DST_PATH="$LAUNCH_AGENTS_DIR/$PLIST_NAME"

BIN_NAME="betterdisplay-kvm"

if command -v "$(pwd)/target/release/$BIN_NAME" >/dev/null 2>&1; then
  BIN_PATH="$(pwd)/target/release/$BIN_NAME"
elif command -v "$(pwd)/target/debug/$BIN_NAME" >/dev/null 2>&1; then
  BIN_PATH="$(pwd)/target/debug/$BIN_NAME"
elif command -v "$BIN_NAME" >/dev/null 2>&1; then
  BIN_PATH="$(command -v "$BIN_NAME")"
else
  echo "Binary $BIN_NAME not found. Build it first: cargo build --release" >&2
  exit 1
fi

mkdir -p "$LAUNCH_AGENTS_DIR"

TMP_PLIST="$(mktemp)"
sed "s#__BIN_PATH__#${BIN_PATH}#g" "$PLIST_SRC_PATH" > "$TMP_PLIST"

mv "$TMP_PLIST" "$PLIST_DST_PATH"

launchctl unload -w "$PLIST_DST_PATH" 2>/dev/null || true
launchctl load -w "$PLIST_DST_PATH"

echo "Installed and loaded launch agent: $PLIST_DST_PATH"
echo "Logs: ~/Library/Logs/betterdisplay-kvm.*.log"


