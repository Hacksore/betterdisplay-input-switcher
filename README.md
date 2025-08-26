# betterdisplay-kvm

**tl;dr**: I press one button to switch from my MacBook to my Gaming PC

[Writing about this in notion](https://www.notion.so/boult/BetterDisplay-KVM-259adc9a945c80a196f6db8f52407a8e).

### Development

Run the program in watch mode if you have `cargo-watch` installed.

```sh
RUST_LOG="betterdisplay_kvm" cargo watch -x run
```

Otherwise, do this.

```sh
RUST_LOG="betterdisplay_kvm" cargo run
```

### Installation

#### Option 1: Use the Release Installer (Recommended)

Download the latest `.pkg` from the releases page and run it. This will:
- Install the binary to `/usr/local/libexec/betterdisplay-kvm/`
- Install the LaunchAgent to `/Library/LaunchAgents/`
- Automatically load and enable the service for your user account

#### Option 2: Build from Source

1) Build the installer package

```sh
scripts/build-pkg.sh
```

2) Run the generated installer from `dist/`

This creates the same installation as the release installer.

#### Option 3: Manual LaunchAgent Setup

For development or custom setups, you can manually install the LaunchAgent:

1) Build the binary

```sh
cargo build --release
```

2) Install and load the LaunchAgent

```sh
mkdir -p ~/Library/LaunchAgents
sed "s#__BIN_PATH__#$(pwd)/target/release/betterdisplay-kvm#g" \
  contrib/launch/com.github.hacksore.betterdisplay-kvm.plist \
  > ~/Library/LaunchAgents/com.github.hacksore.betterdisplay-kvm.plist

launchctl load -w ~/Library/LaunchAgents/com.github.hacksore.betterdisplay-kvm.plist
```

### Configuration

Edit config at `~/.config/betterdisplay-kvm/config.toml`.

### Service Management

To restart the service:

```sh
launchctl unload -w ~/Library/LaunchAgents/com.github.hacksore.betterdisplay-kvm.plist
launchctl load -w ~/Library/LaunchAgents/com.github.hacksore.betterdisplay-kvm.plist
```

To remove:

```sh
launchctl unload -w ~/Library/LaunchAgents/com.github.hacksore.betterdisplay-kvm.plist
rm ~/Library/LaunchAgents/com.github.hacksore.betterdisplay-kvm.plist
```

Logs are written to:
- `~/Library/Logs/betterdisplay-kvm.out.log`
- `~/Library/Logs/betterdisplay-kvm.err.log`

### Distribute as a macOS .pkg

Build a signed or unsigned pkg that installs the binary to `/usr/local/libexec/betterdisplay-kvm/` and a LaunchAgent to `/Library/LaunchAgents/`, and bootstraps it for the current console user on install.

```sh
scripts/build-pkg.sh
```

Output will be under `dist/` with the version inferred from `Cargo.toml`.

Note: For notarization and distribution, you'll want to sign the binary and pkg. This script produces an unsigned pkg suited for local installs and testing.

### Roadmap
- [x] add a config system so others can use it
- [ ] add some setup guide for adding a launch agent
- [ ] publish it to crates.io so you can just install from there
- [ ] flaky: fix bug where audio interface on mac might not go back to correct one
- [ ] optimize for performance somehow so this doesnt waste cpu cycles
