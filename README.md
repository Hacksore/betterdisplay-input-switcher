# BetterDisplay Input Switcher

A Rust application that automatically monitors USB devices and switches BetterDisplay inputs when specific devices connect or disconnect.

## Features

- **USB Device Monitoring**: Continuously monitors USB devices for connection/disconnection events
- **Configurable Device IDs**: Set vendor ID and device ID as strings in a configuration file
- **BetterDisplay Integration**: Automatically runs BetterDisplay CLI commands when devices change state
- **Cross-platform**: Currently optimized for macOS (uses `system_profiler`)

## Configuration

The application uses a `config.toml` file for configuration. You can modify the following values:

```toml
# USB Device Information
vendor_id = "05ac"        # Vendor ID as a string
device_id = "12a8"        # Device ID as a string

# BetterDisplay DDC Commands
disconnect_ddc = "18"     # DDC value when device disconnects
connect_ddc = "15"        # DDC value when device connects
```

### Finding Your Device IDs

To find the vendor ID and device ID for your USB device:

1. **On macOS**: Use the System Information app or run:
   ```bash
   system_profiler SPUSBDataType
   ```

2. **Look for entries like**:
   ```
   Product ID: 0x12a8
   Vendor ID: 0x05ac (Apple Inc.)
   ```

3. **Convert to strings**: Remove the `0x` prefix and use the hexadecimal values as strings

## Installation

1. **Install Rust** (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Clone and build**:
   ```bash
   git clone <repository-url>
   cd betterdisplay-input-switcher
   cargo build --release
   ```

3. **Install BetterDisplay CLI** (if not already installed):
   - Download BetterDisplay from the App Store
   - The CLI should be available as `betterdisplaycli`

## Usage

1. **Configure your device IDs** in `config.toml`

2. **Run the application**:
   ```bash
   cargo run --release
   ```

3. **The application will**:
   - Load configuration from `config.toml`
   - Start monitoring USB devices
   - Automatically run BetterDisplay commands when your device connects/disconnects

## Example Use Cases

- **Dock-based workflow**: Automatically switch to external display when laptop is docked
- **Peripheral switching**: Change input when specific keyboard/mouse connects
- **Workstation setup**: Switch between different input sources based on connected devices

## Troubleshooting

### BetterDisplay CLI not found
Make sure BetterDisplay is installed and the CLI is available in your PATH.

### Device not detected
- Verify the vendor ID and device ID are correct
- Check that the device is actually connecting/disconnecting
- Ensure the device appears in `system_profiler SPUSBDataType`

### Permission issues
The application needs access to USB device information. On macOS, this should work by default.

## Development

The application is built with:
- **Rust** for performance and safety
- **Cross-platform USB monitoring** (currently macOS optimized)
- **Configuration file support** (TOML and JSON)
- **Thread-safe event handling**

## License

[Add your license information here]
