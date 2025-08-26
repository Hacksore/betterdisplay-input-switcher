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

### Install as a macOS Launch Agent

You can run this in the background at login using a user LaunchAgent.

#### Automated install

1) Build the binary

```sh
cargo build --release
```

2) Install and load the LaunchAgent

```sh
scripts/install-launch-agent.sh
```

This will:
- Place a plist at `~/Library/LaunchAgents/com.github.hacksore.betterdisplay-kvm.plist`
- Point it to your built binary
- Load it with `launchctl`

Logs:
- `~/Library/Logs/betterdisplay-kvm.out.log`
- `~/Library/Logs/betterdisplay-kvm.err.log`

To restart after changes:

```sh
launchctl unload -w ~/Library/LaunchAgents/com.github.hacksore.betterdisplay-kvm.plist
launchctl load -w ~/Library/LaunchAgents/com.github.hacksore.betterdisplay-kvm.plist
```

To remove:

```sh
launchctl unload -w ~/Library/LaunchAgents/com.github.hacksore.betterdisplay-kvm.plist
rm ~/Library/LaunchAgents/com.github.hacksore.betterdisplay-kvm.plist
```

#### Manual install

1) Build the binary and find its absolute path

```sh
cargo build --release
BIN_PATH="$(pwd)/target/release/betterdisplay-kvm"
```

2) Copy the template and set the binary path

```sh
mkdir -p ~/Library/LaunchAgents
sed "s#__BIN_PATH__#${BIN_PATH}#g" contrib/launch/com.github.hacksore.betterdisplay-kvm.plist \
  > ~/Library/LaunchAgents/com.github.hacksore.betterdisplay-kvm.plist
```

3) Load it

```sh
launchctl load -w ~/Library/LaunchAgents/com.github.hacksore.betterdisplay-kvm.plist
```

Tip: Edit config at `~/.config/betterdisplay-kvm/config.toml`.

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
