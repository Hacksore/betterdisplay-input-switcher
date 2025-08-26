use futures_lite::stream::StreamExt;
use log::{debug, info};
use nusb::MaybeFuture;
use nusb::hotplug::HotplugEvent;
use std::{collections::HashMap, process::Command};

// NOTE: this is my device used to trigger input switching
pub const USB_DEVICE_ID: &str = "046d:c547";

// TOOD: this be configurable via something like a json or toml file
enum MonitorInput {
  // THIS IS DisplayPort1
  MacBook = 15,
  // THIS IS HDMI2 
  GamingPC = 18,
}

fn set_input(input: MonitorInput) {
  // TODO: error handle if they don't have betterdisplaycli installed or on their PATH
  Command::new("betterdisplaycli")
    .args([
      "set",
      &format!("--ddc={}", input as u16),
      "--vcp=inputSelect",
    ])
    .spawn()
    .expect("failed to execute process");
}

fn on_connect() {
  info!("switch input to the MacBook");
  set_input(MonitorInput::MacBook);
}

fn on_disconnect() {
  info!("switch input to the Gaming PC");
  set_input(MonitorInput::GamingPC);
}

fn main() -> anyhow::Result<()> {
  env_logger::init();
  debug!("Starting betterdisplay");
  let mut devices: HashMap<nusb::DeviceId, (u16, u16)> = HashMap::new();

  // we need to enumerate all devices and make sure they are cached
  // otherwise we won't get disconnect events for devices that were
  debug!("Enumerate all USB devices");
  for info in nusb::list_devices().wait().unwrap() {
    let id = info.id();
    let vendor = info.vendor_id();
    let product = info.product_id();
    let device_str = format!("{:04x}:{:04x}", vendor, product);
    devices.insert(id, (vendor, product));

    debug!("Found USB device: {}", device_str);

    // TODO: is this working?
    // if we see the device on startup, switch input to MacBook
    if device_str == USB_DEVICE_ID {
      set_input(MonitorInput::MacBook);
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

          if device_str == USB_DEVICE_ID {
            debug!("Connected to USB_DEVICE_ID device: {}", device_str);
            on_connect();
          }

          // Cache vendor/product by DeviceId
          devices.insert(id, (vendor, product));
        }
        HotplugEvent::Disconnected(id) => {
          if let Some((vendor, product)) = devices.remove(&id) {
            let device_str = format!("{:04x}:{:04x}", vendor, product);

            if device_str == USB_DEVICE_ID {
              debug!("Disconnected USB_DEVICE_ID USB device: {}", device_str);
              on_disconnect();
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
