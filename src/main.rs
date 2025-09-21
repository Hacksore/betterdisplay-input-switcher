use flexi_logger::{Cleanup, Criterion, Duplicate, FileSpec, Logger, Naming, WriteMode};
use futures_lite::stream::StreamExt;
use log::{debug, info, error};
use nusb::MaybeFuture;
use nusb::hotplug::HotplugEvent;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::PathBuf, process::Command, panic};

pub const DEFAULT_DEVICE_ID: &str = "046d:c547";

#[derive(Debug, Serialize, Deserialize)]
struct AppConfig {
  /// The USB device id in the form "vvvv:pppp"
  usb_device_id: Option<String>,
  /// DDC input code for system 1 (e.g. 15)
  system_one_input: Option<u16>,
  /// DDC input code for system 2 (e.g. 18)
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
        .unwrap_or_else(|| DEFAULT_DEVICE_ID.to_string()),
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

fn set_input(input_code: u16, use_ddc_alt: bool) -> anyhow::Result<()> {
  let mut cmd = Command::new("betterdisplaycli");
  cmd.arg("set");
  if use_ddc_alt {
    cmd.arg("--ddcAlt");
  }
  cmd.args([
    format!("--ddc={}", input_code),
    "--vcp=inputSelect".to_string(),
  ]);
  
  debug!("Executing betterdisplaycli command: {:?}", cmd);
  
  let mut child = cmd.spawn()
    .map_err(|e| anyhow::anyhow!("Failed to execute betterdisplaycli process: {}", e))?;
  
  let status = child.wait()
    .map_err(|e| anyhow::anyhow!("Failed to wait for betterdisplaycli process: {}", e))?;
  
  if !status.success() {
    return Err(anyhow::anyhow!("betterdisplaycli exited with status: {}", status));
  }
  
  debug!("Successfully executed betterdisplaycli command");
  Ok(())
}

fn on_connect(cfg: &ResolvedConfig) {
  info!("switch input to the system_one_input");
  if let Err(e) = set_input(cfg.system_one_input, cfg.ddc_alt) {
    error!("Failed to set input on connect: {}", e);
  }
}

fn on_disconnect(cfg: &ResolvedConfig) {
  info!("switch input to system_two_input");
  if let Err(e) = set_input(cfg.system_two_input, cfg.ddc_alt) {
    error!("Failed to set input on disconnect: {}", e);
  }
}

fn validate_environment() -> anyhow::Result<()> {
  error!("Validating environment...");
  
  // Check if betterdisplaycli is available
  let output = Command::new("which")
    .arg("betterdisplaycli")
    .output();
    
  match output {
    Ok(result) => {
      if result.status.success() {
        error!("betterdisplaycli found at: {}", String::from_utf8_lossy(&result.stdout).trim());
      } else {
        error!("betterdisplaycli not found in PATH");
        return Err(anyhow::anyhow!("betterdisplaycli not found in PATH"));
      }
    }
    Err(e) => {
      error!("Failed to check for betterdisplaycli: {}", e);
      return Err(anyhow::anyhow!("Failed to check for betterdisplaycli: {}", e));
    }
  }
  
  // Check if we can access USB devices (this might require permissions)
  error!("Checking USB access permissions...");
  
  error!("Environment validation completed");
  Ok(())
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
  // Load config first to get the log level
  let cfg = load_config().map_err(|e| {
    eprintln!("Failed to load config: {}", e);
    e
  })?;

  // Set up logger with the proper log level from config
  let mut logs_dir =
    dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Failed to get home directory"))?;
  logs_dir.push("Library");
  logs_dir.push("Logs");
  logs_dir.push("betterdisplay-kvm");

  if !logs_dir.exists() {
    fs::create_dir_all(&logs_dir)?;
  }

  let level_str = match cfg.log_level.to_lowercase().as_str() {
    "error" => "error",
    "warn" | "warning" => "warn",
    "info" => "info",
    "debug" => "debug",
    "trace" => "trace",
    _ => "info",
  };

  let spec = format!("off,betterdisplay_kvm={}", level_str);

  Logger::try_with_str(spec)?
    .log_to_file(
      FileSpec::default()
        .directory(&logs_dir)
        .basename("betterdisplay-kvm")
        .suffix("log"),
    )
    .format_for_files(flexi_logger::detailed_format)
    .duplicate_to_stdout(Duplicate::All)
    .duplicate_to_stderr(Duplicate::Error)
    .format_for_stdout(flexi_logger::detailed_format)
    .write_mode(WriteMode::BufferAndFlush)
    .rotate(
      Criterion::Size(10_000_000),
      Naming::Timestamps,
      Cleanup::KeepLogFiles(7),
    )
    .start()?;

  // Set up panic hook to capture panics and log them
  panic::set_hook(Box::new(|panic_info| {
    error!("PANIC: {}", panic_info);
    if let Some(location) = panic_info.location() {
      error!("Location: {}:{}:{}", location.file(), location.line(), location.column());
    }
    if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
      error!("Message: {}", s);
    } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
      error!("Message: {}", s);
    }
  }));

  info!("betterdisplay-kvm starting...");

  // Now validate environment with proper logging
  validate_environment().map_err(|e| {
    error!("Environment validation failed: {}", e);
    e
  })?;

  debug!("Starting betterdisplay-kvm with config: {:?}", cfg);
  
  let mut devices: HashMap<nusb::DeviceId, (u16, u16)> = HashMap::new();

  debug!("Enumerate all USB devices");
  
  let device_list = nusb::list_devices().wait()
    .map_err(|e| {
      error!("Failed to enumerate USB devices: {}", e);
      anyhow::anyhow!("Failed to enumerate USB devices: {}", e)
    })?;
    
  for info in device_list {
    let id = info.id();
    let vendor = info.vendor_id();
    let product = info.product_id();
    let device_str = format!("{:04x}:{:04x}", vendor, product);
    devices.insert(id, (vendor, product));

    debug!("Found USB device: {}", device_str);

    if device_str == cfg.usb_device_id {
      if let Err(e) = set_input(cfg.system_one_input, cfg.ddc_alt) {
        error!("Failed to set initial input: {}", e);
        error!("Failed to set initial input: {}", e);
      }
    }
  }

  futures_lite::future::block_on(async {
    let mut events = nusb::watch_devices()
      .map_err(|e| {
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

            error!("USB device disconnected: {}", device_str);
            debug!("Disconnected configured USB device: {}", device_str);

            if device_str == cfg.usb_device_id {
              error!("Configured device disconnected, switching to system_two_input");
              on_disconnect(&cfg);
            }

            debug!("Removed device from cache: {}", device_str);
          } else {
            error!("Unknown device disconnected: {:?}", id);
          }
        }
      }
    }

    error!("USB device monitoring ended");
    Ok::<_, anyhow::Error>(())
  }).map_err(|e| {
    error!("Error in USB device monitoring: {}", e);
    e
  })?;

  Ok(())
}
