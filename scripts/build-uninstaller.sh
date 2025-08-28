#!/bin/sh
set -euo pipefail

echo "Building uninstaller package..."

PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DIST_DIR="$PROJECT_ROOT/dist"
SCRIPTS_DIR="$PROJECT_ROOT/pkg/scripts"
IDENTIFIER="com.github.hacksore.betterdisplay-kvm-uninstaller"
VERSION="1.0.0"
PKG_PATH="$DIST_DIR/betterdisplay-kvm-uninstaller-$VERSION.pkg"

mkdir -p "$DIST_DIR" "$SCRIPTS_DIR"

# Create postinstall script that removes everything
cat > "$SCRIPTS_DIR/postinstall" << 'EOS'
#!/bin/sh
set -e

echo "Uninstalling betterdisplay-kvm..."

# Find current console user UID
CONSOLE_UID=$(stat -f %u /dev/console 2>/dev/null || true)
if [ -n "$CONSOLE_UID" ] && [ "$CONSOLE_UID" -gt 0 ]; then
  # Unload and disable the LaunchAgent
  launchctl bootout gui/$CONSOLE_UID /Library/LaunchAgents/com.github.hacksore.betterdisplay-kvm.plist 2>/dev/null || true
fi

# Remove the binary
rm -f /usr/local/libexec/betterdisplay-kvm/betterdisplay-kvm
rmdir /usr/local/libexec/betterdisplay-kvm 2>/dev/null || true

# Remove the LaunchAgent plist
rm -f /Library/LaunchAgents/com.github.hacksore.betterdisplay-kvm.plist

# Remove logs for all users
for USER_HOME in /Users/*; do
  if [ -d "$USER_HOME" ] && [ "$(basename "$USER_HOME")" != "Shared" ]; then
    rm -f "$USER_HOME/Library/Logs/betterdisplay-kvm.out.log"
    rm -f "$USER_HOME/Library/Logs/betterdisplay-kvm.err.log"
  fi
done

# Also check for root user logs
rm -f /var/root/Library/Logs/betterdisplay-kvm.out.log
rm -f /var/root/Library/Logs/betterdisplay-kvm.err.log

echo "Uninstallation complete!"
exit 0
EOS
chmod 755 "$SCRIPTS_DIR/postinstall"

echo "Building uninstaller pkg at $PKG_PATH ..."
pkgbuild \
  --scripts "$SCRIPTS_DIR" \
  --identifier "$IDENTIFIER" \
  --version "$VERSION" \
  --nopayload \
  "$PKG_PATH"

echo "Done: $PKG_PATH"
echo "This package will uninstall betterdisplay-kvm when run."
