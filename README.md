# BetterDisplay KVM

A Rust-based KVM switch utility for BetterDisplay that utilizes the [`betterdisplaycli`](https://github.com/waydabber/betterdisplaycli).

## How it works

This works by using the BetterDisplay app and CLI to issue commands to your monitor when a configured USB device is connected or disconnected via the `betterdisplay-kvm` Rust program. It uses the [DDC/CI](https://en.wikipedia.org/wiki/Display_Data_Channel) protocol to send commands directly to your monitor. 

With a single press of a button, you can switch to your gaming PC or MacBook seamlessly.

## Diagram

![diagram](./betterdisplay-kvm-diagram.png)

## Why not use a KVM?

Because they don’t support high refresh rates without spending an ungodly amount of money.

## Config

```toml
# the USB device you'd like to watch for
usb_device_id = "046d:c547"
# ID that betterdisplaycli uses to configure input
system_one_input = 15
# ID that betterdisplaycli uses to configure input
system_two_input = 18
# log level
log_level = "debug"
# if you use an LG monitor that doesn't follow the spec, this might work if you enable it
ddc_alt = false
```

## Development

```bash
RUST_LOG=debug cargo watch -x run
```

## Install

Run `./install.sh`, and it will install a LaunchAgent and start the program.

## Uninstall

Run `./uninstall.sh`, and it will remove the program and clean everything up.

### Roadmap
- [x] Add a config system so others can use it
- [x] Add a setup guide for adding a launch agent
- [x] Fix the hardcoded bin path to `betterdisplaycli` from Homebrew
- [x] Publish it to crates.io so you can install it from there
- [ ] Codesign the binary for people who want to use it outside of Homebrew
