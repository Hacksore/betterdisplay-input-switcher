# BetterDisplay KVM

A Rust-based KVM switch utility for BetterDisplay.

## Building

### Installer Package

To build the installer package:

```bash
./scripts/build-installer.sh
```

This will create a `.pkg` file in the `dist/` directory that can be installed on macOS.

### Uninstaller Package

To build the uninstaller package:

```bash
./scripts/build-uninstaller.sh
```

This will create an uninstaller `.pkg` file in the `dist/` directory that will remove:
- The binary from `/usr/local/libexec/betterdisplay-kvm/`
- The LaunchAgent from `/Library/LaunchAgents/`
- All log files from user directories

## Installation

1. Run the installer package: `sudo installer -pkg betterdisplay-kvm-*.pkg -target /`
2. The LaunchAgent will be automatically installed and started

## Uninstallation

1. Run the uninstaller package: `sudo installer -pkg betterdisplay-kvm-uninstaller-*.pkg -target /`
2. All components will be removed automatically

## Manual Uninstallation

If you need to uninstall manually:

```bash
# Stop and remove the LaunchAgent
launchctl bootout gui/$UID /Library/LaunchAgents/com.github.hacksore.betterdisplay-kvm.plist

# Remove files
sudo rm -rf /usr/local/libexec/betterdisplay-kvm
sudo rm -f /Library/LaunchAgents/com.github.hacksore.betterdisplay-kvm.plist

# Remove logs
rm -f ~/Library/Logs/betterdisplay-kvm.*.log
```
