# BetterDisplay KVM

A Rust-based KVM switch utility for BetterDisplay.


## Config

```toml
# the USB device you'd like watch for
usb_device_id = "046d:c547"
# id that betterdisplaycli uses to configure input
system_one_input = 15
# id that betterdisplaycli uses to configure input
system_two_input = 18
# log level
log_level = "debug"
# if you use an lg monitor that doesnt follow the spec this might work if you enable it
ddc_alt = false

```

## Development

```
RUST_LOG=debug cargo watch -x run
```

## Install

Run the `./install.sh` and it will install a LaunchAgent and start the program.

## Uninstall

Run the `./uninstall.sh` and it remove the program and clean everything up.

### Roadmap
- [x] add a config system so others can use it
- [x] add some setup guide for adding a launch agent
- [ ] publish it to crates.io so you can just install from there
- [ ] flaky: fix bug where audio interface on mac might not go back to correct one
- [ ] optimize for performance somehow so this doesnt waste cpu cycles
