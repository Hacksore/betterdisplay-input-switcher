use futures_lite::stream::StreamExt;
use log::{debug, info};
use flexi_logger::{Cleanup, Criterion, Duplicate, FileSpec, Logger, Naming, WriteMode};
use nusb::MaybeFuture;
use nusb::hotplug::HotplugEvent;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::PathBuf, process::Command};

#[derive(Debug, Serialize, Deserialize)]
struct AppConfig {
  /// The USB device id in the form "vvvv:pppp"
  usb_device_id: Option<String>,
  /// DDC input code for computer 1 (e.g. 15)
  system_one_input: Option<u16>,
  /// DDC input code for computer 2 (e.g. 18)
  system_two_input: Option<u16>,
  /// Log level: "error", "warn", "info", "debug", "trace"
  log_level: Option<String>,
  /// Enable alternative DDC flag for LG monitors (adds --ddcAlt)
  ddc_alt: Option<bool>,
}

impl AppConfig {
  fn with_defaults(self) -> ResolvedConfig {
    ResolvedConfig {
      usb_device_id: self
        .usb_device_id
        .unwrap_or_else(|| "046d:c547".to_string()),
      system_one_input: self.system_one_input.unwrap_or(15),
      system_two_input: self.system_two_input.unwrap_or(18),
      log_level: self.log_level.unwrap_or_else(|| "info".to_string()),
      ddc_alt: self.ddc_alt.unwrap_or(false),
    }
  }
}

#[derive(Debug, Clone, Serialize)]
struct ResolvedConfig {
  usb_device_id: String,
  system_one_input: u16,
  system_two_input: u16,
  log_level: String,
  ddc_alt: bool,
}

fn set_input(input_code: u16, use_ddc_alt: bool) {
  let mut cmd = Command::new("betterdisplaycli");
  cmd.arg("set");
  if use_ddc_alt {
    cmd.arg("--ddcAlt");
  }
  cmd.args([
    format!("--ddc={}", input_code),
    "--vcp=inputSelect".to_string(),
  ]);
  cmd
    .spawn()
    .expect("failed to execute betterdisplaycli process");
}

fn on_connect(cfg: &ResolvedConfig) {
  info!("switch input to the system_one_input");
  set_input(cfg.system_one_input, cfg.ddc_alt);
}

fn on_disconnect(cfg: &ResolvedConfig) {
  info!("switch input to system_two_input");
  set_input(cfg.system_two_input, cfg.ddc_alt);
}

fn load_config() -> anyhow::Result<ResolvedConfig> {
  let oshome = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Failed to get home directory"))?;
  let mut config_path = PathBuf::from(oshome);
  config_path.push(".config");
  config_path.push("betterdisplay-kvm");
  config_path.push("config.toml");

  let builder =
    config::Config::builder().add_source(config::File::from(config_path.clone()).required(false));

  let cfg: AppConfig = builder.build()?.try_deserialize()?;

  // If the config file does not exist, create the directory and write defaults
  if !config_path.exists() {
    if let Some(parent) = config_path.parent() {
      if !parent.exists() {
        fs::create_dir_all(parent)?;
      }
    }
    let resolved = cfg.with_defaults();
    fs::write(&config_path, toml::to_string_pretty(&resolved)?)?;
    return Ok(resolved);
  }

  Ok(cfg.with_defaults())
}

fn main() -> anyhow::Result<()> {
  let cfg = load_config()?;

  // Initialize file-based logging under ~/Library/Logs/betterdisplay-kvm
  let mut logs_dir = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Failed to get home directory"))?;
  logs_dir.push("Library");
  logs_dir.push("Logs");
  logs_dir.push("betterdisplay-kvm");

  // Ensure log directory exists
  if !logs_dir.exists() {
    fs::create_dir_all(&logs_dir)?;
  }

  // Map config log level
  let level_str = match cfg.log_level.to_lowercase().as_str() {
    "error" => "error",
    "warn" | "warning" => "warn",
    "info" => "info",
    "debug" => "debug",
    "trace" => "trace",
    _ => "info",
  };

  // Only enable logs for this crate by default, others off
  let spec = format!("off,betterdisplay_kvm={}", level_str);

  Logger::try_with_str(spec)?
    .log_to_file(FileSpec::default()
      .directory(&logs_dir)
      .basename("betterdisplay-kvm")
      .suffix("log"))
    .format_for_files(flexi_logger::detailed_format)
    .duplicate_to_stdout(Duplicate::All)
    .duplicate_to_stderr(Duplicate::Error)
    .format_for_stdout(flexi_logger::detailed_format)
    .write_mode(WriteMode::BufferAndFlush)
    .rotate(Criterion::Size(10_000_000), Naming::Timestamps, Cleanup::KeepLogFiles(7))
    .start()?;

  debug!("Starting betterdisplay-kvm with config: {:?}", cfg);
  let mut devices: HashMap<nusb::DeviceId, (u16, u16)> = HashMap::new();

  debug!("Enumerate all USB devices");
  for info in nusb::list_devices().wait().unwrap() {
    let id = info.id();
    let vendor = info.vendor_id();
    let product = info.product_id();
    let device_str = format!("{:04x}:{:04x}", vendor, product);
    devices.insert(id, (vendor, product));

    debug!("Found USB device: {}", device_str);

    // if we see the device on startup, switch input to system_one_input
    if device_str == cfg.usb_device_id {
      set_input(cfg.system_one_input, cfg.ddc_alt);
    }
  }

  // NOTE: handle hotswapping when things plug/unplug
  futures_lite::future::block_on(async {
    let mut events = nusb::watch_devices()?;

    while let Some(event) = events.next().await {
      match event {
        HotplugEvent::Connected(info) => {
          let id = info.id();
          let vendor = info.vendor_id();
          let product = info.product_id();
          let device_str = format!("{:04x}:{:04x}", vendor, product);

          if device_str == cfg.usb_device_id {
            debug!("Connected to configured USB device: {}", device_str);
            on_connect(&cfg);
          }

          // Cache vendor/product by DeviceId
          devices.insert(id, (vendor, product));
        }
        HotplugEvent::Disconnected(id) => {
          if let Some((vendor, product)) = devices.remove(&id) {
            let device_str = format!("{:04x}:{:04x}", vendor, product);

            if device_str == cfg.usb_device_id {
              debug!("Disconnected configured USB device: {}", device_str);
              on_disconnect(&cfg);
            }

            // remove from cache since they will be cached on connect
            devices.remove(&id);
            debug!("Removed device from cache: {}", device_str);
          }
        }
      }
    }

    Ok::<_, anyhow::Error>(())
  })?;

  Ok(())
}
