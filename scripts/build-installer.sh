#!/bin/sh
set -euo pipefail

echo "cleaning old build..."
rm -rf pkg/ dist/ target/

PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DIST_DIR="$PROJECT_ROOT/dist"
PKG_ROOT="$PROJECT_ROOT/pkg/root"
SCRIPTS_DIR="$PROJECT_ROOT/pkg/scripts"
IDENTIFIER="com.github.hacksore.betterdisplay-kvm"
BIN_INSTALL_DIR="/usr/local/libexec/betterdisplay-kvm"
BIN_NAME="betterdisplay-kvm"
BIN_INSTALL_PATH="$BIN_INSTALL_DIR/$BIN_NAME"
PLIST_NAME="$IDENTIFIER.plist"

mkdir -p "$DIST_DIR" "$PKG_ROOT$BIN_INSTALL_DIR" "$PKG_ROOT/Library/LaunchAgents" "$SCRIPTS_DIR"

# Build release binary
echo "Building release binary..."
cargo build --release --manifest-path "$PROJECT_ROOT/Cargo.toml"

# Determine version from Cargo.toml
VERSION=$(grep -E '^version\s*=\s*"' "$PROJECT_ROOT/Cargo.toml" | head -1 | sed -E 's/.*"([^"]+)".*/\1/')
PKG_PATH="$DIST_DIR/$BIN_NAME-$VERSION.pkg"

# Copy binary into payload
cp "$PROJECT_ROOT/target/release/$BIN_NAME" "$PKG_ROOT$BIN_INSTALL_PATH"
chmod 755 "$PKG_ROOT$BIN_INSTALL_PATH"

# Prepare LaunchAgent plist for system-wide location
SED_PLIST_TMP="$(mktemp)"
sed "s#__BIN_PATH__#${BIN_INSTALL_PATH}#g" \
  "$PROJECT_ROOT/contrib/launch/$PLIST_NAME" > "$SED_PLIST_TMP"
mv "$SED_PLIST_TMP" "$PKG_ROOT/Library/LaunchAgents/$PLIST_NAME"

# Create postinstall script to bootstrap the LaunchAgent for console user
cat > "$SCRIPTS_DIR/postinstall" << 'EOS'
#!/bin/sh
set -e

echo "Installing betterdisplay-kvm LaunchAgent..."

SYSTEM_PLIST="/Library/LaunchAgents/com.github.hacksore.betterdisplay-kvm.plist"

# Ensure correct permissions on system plist
chown root:wheel "$SYSTEM_PLIST" || true
chmod 644 "$SYSTEM_PLIST" || true

# Find current console user UID
CONSOLE_UID=$(stat -f %u /dev/console 2>/dev/null || true)
if [ -n "$CONSOLE_UID" ] && [ "$CONSOLE_UID" -gt 0 ]; then
  echo "Setting up LaunchAgent for user $CONSOLE_UID..."
  
  # Get user home directory
  USER_HOME=$(dscl . -read /Users/$(stat -f %Su /dev/console) NFSHomeDirectory | awk '{print $2}')
  
  if [ -n "$USER_HOME" ] && [ -d "$USER_HOME" ]; then
    # Create user's LaunchAgents directory if it doesn't exist
    USER_LAUNCHAGENTS="$USER_HOME/Library/LaunchAgents"
    mkdir -p "$USER_LAUNCHAGENTS"
    
    # Copy the plist to user's LaunchAgents directory
    USER_PLIST="$USER_LAUNCHAGENTS/com.github.hacksore.betterdisplay-kvm.plist"
    cp "$SYSTEM_PLIST" "$USER_PLIST"
    
    # Set correct ownership for user's copy
    chown "$CONSOLE_UID:staff" "$USER_PLIST"
    chmod 644 "$USER_PLIST"
    
    # Unload if already loaded, then bootstrap and enable
    launchctl bootout gui/$CONSOLE_UID "$USER_PLIST" 2>/dev/null || true
    launchctl bootstrap gui/$CONSOLE_UID "$USER_PLIST" || true
    launchctl enable gui/$CONSOLE_UID/com.github.hacksore.betterdisplay-kvm 2>/dev/null || true
    echo "LaunchAgent installed and enabled successfully!"
  else
    echo "Warning: Could not determine user home directory, LaunchAgent will need to be loaded manually"
  fi
else
  echo "Warning: Could not determine console user, LaunchAgent will need to be loaded manually"
fi

exit 0
EOS
chmod 755 "$SCRIPTS_DIR/postinstall"

echo "Building pkg at $PKG_PATH ..."
pkgbuild \
  --root "$PKG_ROOT" \
  --scripts "$SCRIPTS_DIR" \
  --identifier "$IDENTIFIER" \
  --version "$VERSION" \
  --nopayload \
  "$PKG_PATH"

echo "Done: $PKG_PATH"
