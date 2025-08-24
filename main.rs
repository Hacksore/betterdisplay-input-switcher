use futures_lite::stream;
use nusb::hotplug::HotplugEvent;

fn main() {
    env_logger::init();
    for event in stream::block_on(nusb::watch_devices().unwrap()) {
        // TODO: match events over HotplugEvent::Connected and HotplugEvent::Disconnected
        match event {
            HotplugEvent::Connected(device_info) => {
                print_device_info(&device_info);
            }
            HotplugEvent::Disconnected(device_id) => {
                print_device_id(&device_id);
            }
        }
    }
}

fn print_device_id(device_id: &nusb::DeviceId) {
    println!("dis: {:?}", device_id)
}

fn print_device_info(d: &nusb::DeviceInfo) {
    println!("c: {:?}, {:?}, ({:?})", d.manufacturer_string(), d.product_string(), d.id());
}
