mod app;
mod device_manager;
mod utils;

use clap::{CommandFactory, Parser};
use log::error;
use utils::handle_launch_agent;

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

  // Initialize the application
  let app = app::App::initialize()?;

  // Create device manager and start monitoring
  let mut device_manager = device_manager::DeviceManager::new(app.config().clone());

  // Enumerate existing devices
  device_manager.enumerate_devices()?;

  // Start monitoring for device changes
  futures_lite::future::block_on(async { device_manager.monitor_devices().await }).map_err(
    |e| {
      error!("Error in USB device monitoring: {}", e);
      e
    },
  )?;

  Ok(())
}
