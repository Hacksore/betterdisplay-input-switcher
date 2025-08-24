use futures_lite::stream;
use nusb::hotplug::HotplugEvent;

pub const LOGI_USB_ID: &str = "4294994929";

// Device connected: 1133:50503 (Some("USB Receiver") - Some

fn main() {
    env_logger::init();
    for event in stream::block_on(nusb::watch_devices().unwrap()) {
        // TODO: match events over HotplugEvent::Connected and HotplugEvent::Disconnected
        match event {
            HotplugEvent::Connected(d) => {
                let id = format!("{:?}", d.id());
                let product = format!("{:?} - {:?}", d.product_string(), d.manufacturer_string());
                println!("Device Connected: {} ({})", id, product);
            }
            HotplugEvent::Disconnected(d) => {
                let id = format!("{:?}", d);
                println!("Device Disconnected: {}", id);
            }
        }
    }
}

