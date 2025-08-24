use futures_lite::stream::StreamExt;
use nusb::hotplug::HotplugEvent;
use std::collections::HashMap;

pub const LOGI_USB_DEVICES: &str = "046d:c547";

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

                    println!(
                        "Connected: {:04x}:{:04x}, {:?}",
                        vendor, product, info.manufacturer_string()
                    );

                    // Cache vendor/product by DeviceId
                    devices.insert(id, (vendor, product));
                }
                HotplugEvent::Disconnected(id) => {
                    if let Some((vendor, product)) = devices.remove(&id) {
                        println!(
                            "Disconnected: {:04x}:{:04x} (id={:?})",
                            vendor, product, id
                        );
                    } else {
                        println!("Disconnected unknown device: {:?}", id);
                    }
                }
            }
        }

        Ok::<_, anyhow::Error>(())
    })?;

    Ok(())
}
