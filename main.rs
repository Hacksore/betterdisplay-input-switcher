use futures_lite::stream::StreamExt;
use log::debug;
use nusb::hotplug::HotplugEvent;
use std::{collections::HashMap, process::Command};

pub const LOGI_USB_DEVICES: &str = "046d:c547";

fn on_connect() {
  println!("Would switch input to the MacBook");
  // Command::new("betterdisplaycli")
  //   .args(["set", "--ddc=15", "--vcp=inputSelect"])
  //   .spawn()
  //   .expect("failed to execute process");
}

fn on_disconnect() {
  println!("Would switch input to the Gaming PC");
  // Command::new("betterdisplaycli")
  //   .args(["set", "--ddc=18", "--vcp=inputSelect"])
  //   .spawn()
  //   .expect("failed to execute process");
}


fn main() -> anyhow::Result<()> {
  env_logger::init();

  futures_lite::future::block_on(async {
    let mut events = nusb::watch_devices()?;
    let mut devices: HashMap<nusb::DeviceId, (u16, u16)> = HashMap::new();

    while let Some(event) = events.next().await {
      match event {
        HotplugEvent::Connected(info) => {
          let id = info.id();
          let vendor = info.vendor_id();
          let product = info.product_id();
          let device_str = format!("{:04x}:{:04x}", vendor, product);

          if device_str == LOGI_USB_DEVICES {
            debug!("Connected Logitech USB device: {}", device_str);
            on_connect();
          }

          // Cache vendor/product by DeviceId
          devices.insert(id, (vendor, product));
        }
        HotplugEvent::Disconnected(id) => {
          if let Some((vendor, product)) = devices.remove(&id) {
            let device_str = format!("{:04x}:{:04x}", vendor, product);

            if device_str == LOGI_USB_DEVICES {
              debug!("Disconnected Logitech USB device: {}", device_str);
              on_disconnect();
            }
          }
        }
      }
    }

    Ok::<_, anyhow::Error>(())
  })?;

  Ok(())
}
