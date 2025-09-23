mod utils;

use clap::{CommandFactory, Parser};
use futures_lite::stream::StreamExt;
use log::{debug, error, info};
use nusb::MaybeFuture;
use nusb::hotplug::HotplugEvent;
use std::{collections::HashMap, panic};
use utils::{
  get_betterdisplay_path, handle_launch_agent, load_config, on_connect, on_disconnect, set_input,
  setup_logger,
};

/// A KVM switch for BetterDisplay
#[derive(Parser, Debug)]
#[command(name = "betterdisplay-kvm")]
#[command(about = "A KVM switch for BetterDisplay")]
#[command(version)]
#[command(long_about = "BetterDisplay KVM Switch

A daemon that monitors USB device connections and automatically switches
BetterDisplay input sources based on configured USB device events.

This tool requires the --launch flag to run as a long-lived daemon that
monitors USB devices. Use --install to set up the launch agent.")]
struct Cli {
  /// Install the launch agent for automatic startup
  #[arg(
    long,
    help = "Install the macOS launch agent to automatically start the daemon"
  )]
  install: bool,

  /// Run as a long-lived daemon (required for normal operation)
  #[arg(
    long,
    help = "Run as a daemon to monitor USB devices and switch inputs"
  )]
  launch: bool,
}

fn main() -> anyhow::Result<()> {
  let cli = Cli::parse();

  // Handle install flag
  if cli.install {
    handle_launch_agent()?;
    return Ok(());
  }

  // Check if launch flag is provided for long-lived execution
  if !cli.launch {
    // Use clap's built-in help functionality
    let mut cmd = Cli::command();
    cmd.print_help()?;
    eprintln!();
    eprintln!("Note: This program requires --launch to run as a daemon.");
    return Ok(());
  }

  // Load config first to get the log level
  let cfg = load_config().map_err(|e| {
    eprintln!("Failed to load config: {}", e);
    e
  })?;

  // Set up logger with the proper log level from config
  setup_logger(&cfg)?;

  info!("betterdisplay-kvm starting...");

  let betterdisplay_path = get_betterdisplay_path();
  debug!("Found betterdisplaycli at: {:?}", betterdisplay_path);

  // Set up panic hook to capture panics and log them
  panic::set_hook(Box::new(|panic_info| {
    error!("PANIC: {}", panic_info);
    if let Some(location) = panic_info.location() {
      error!(
        "Location: {}:{}:{}",
        location.file(),
        location.line(),
        location.column()
      );
    }
    if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
      error!("Message: {}", s);
    } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
      error!("Message: {}", s);
    }
  }));

  debug!("Starting betterdisplay-kvm with config: {:?}", cfg);

  let mut devices: HashMap<nusb::DeviceId, (u16, u16)> = HashMap::new();

  debug!("Enumerate all USB devices");

  let device_list: Vec<nusb::DeviceInfo> = nusb::list_devices().wait()?.collect();

  for info in device_list {
    let id = info.id();
    let vendor = info.vendor_id();
    let product = info.product_id();
    let manufacturer_name = info.manufacturer_string().unwrap_or("Unknown Manufacturer");
    let device_name = info.product_string().unwrap_or("Unknown Product");
    let device_str = format!("{:04x}:{:04x}", vendor, product);

    devices.insert(id, (vendor, product));

    debug!(
      "{:?} ({:?}): {}",
      device_name, manufacturer_name, device_str
    );

    if device_str == cfg.usb_device_id {
      info!(
        "Configured USB device {}, switching input to {}",
        device_str, cfg.system_one_input
      );
      if let Err(e) = set_input(cfg.system_one_input, cfg.ddc_alt) {
        error!("Failed to set initial input: {}", e);
      }
    }
  }

  futures_lite::future::block_on(async {
    let mut events = nusb::watch_devices().map_err(|e| {
      error!("Failed to start USB device monitoring: {}", e);
      anyhow::anyhow!("Failed to start USB device monitoring: {}", e)
    })?;

    while let Some(event) = events.next().await {
      match event {
        HotplugEvent::Connected(info) => {
          let id = info.id();
          let vendor = info.vendor_id();
          let product = info.product_id();
          let device_str = format!("{:04x}:{:04x}", vendor, product);

          debug!("Connected to configured USB device: {}", device_str);

          if device_str == cfg.usb_device_id {
            on_connect(&cfg);
          }

          devices.insert(id, (vendor, product));
          debug!("Added device to cache: {}", device_str);
        }
        HotplugEvent::Disconnected(id) => {
          if let Some((vendor, product)) = devices.remove(&id) {
            let device_str = format!("{:04x}:{:04x}", vendor, product);
            debug!("Disconnected configured USB device: {}", device_str);

            if device_str == cfg.usb_device_id {
              debug!("Configured device disconnected, switching to system_two_input");
              on_disconnect(&cfg);
            }

            debug!("Removed device from cache: {}", device_str);
          } else {
            error!("Unknown device disconnected: {:?}", id);
          }
        }
      }
    }

    Ok::<_, anyhow::Error>(())
  })
  .map_err(|e| {
    error!("Error in USB device monitoring: {}", e);
    e
  })?;

  Ok(())
}
