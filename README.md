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

### Roadmap
- [ ] add a config system so others can use it
- [ ] add some setup guide for adding a launch agent
- [ ] publish it to crates.io so you can just install from there
- [ ] optimize for performance somehow so this doesnt waste cpu cycles
