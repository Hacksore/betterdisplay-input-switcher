use std::process::Command;
use std::thread;
use std::time::Duration;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::fs;
use serde::{Deserialize, Serialize};

// Configuration structure for USB device monitoring
#[derive(Clone, Serialize, Deserialize)]
struct UsbConfig {
    vendor_id: String,
    device_id: String,
    disconnect_ddc: String,
    connect_ddc: String,
}

impl Default for UsbConfig {
    fn default() -> Self {
        UsbConfig {
            vendor_id: "05ac".to_string(),  // Apple vendor ID (example)
            device_id: "12a8".to_string(),  // Example device ID
            disconnect_ddc: "18".to_string(),
            connect_ddc: "15".to_string(),
        }
    }
}

// USB device information
struct UsbDevice {
    vendor_id: String,
    device_id: String,
    connected: bool,
}

fn main() {
    // Load configuration from file or use defaults
    let config = load_config().unwrap_or_default();
    
    println!("Starting USB monitor for vendor ID: {}, device ID: {}", 
             config.vendor_id, config.device_id);
    println!("Disconnect DDC: {}, Connect DDC: {}", 
             config.disconnect_ddc, config.connect_ddc);

    // Shared state for device connection status
    let device_state = Arc::new(Mutex::new(HashMap::new()));
    let device_state_clone = Arc::clone(&device_state);

    // Channel for communication between threads
    let (tx, rx) = mpsc::channel();

    // Spawn USB monitoring thread
    let config_clone = config.clone();
    thread::spawn(move || {
        monitor_usb_devices(config_clone, tx);
    });

    // Main thread handles device state changes and runs BetterDisplay commands
    handle_device_changes(rx, device_state_clone, config);
}

fn load_config() -> Result<UsbConfig, Box<dyn std::error::Error>> {
    // Try to load from config.toml first
    if let Ok(config_content) = fs::read_to_string("config.toml") {
        let config: UsbConfig = toml::from_str(&config_content)?;
        println!("Loaded configuration from config.toml");
        return Ok(config);
    }
    
    // Try to load from config.json
    if let Ok(config_content) = fs::read_to_string("config.json") {
        let config: UsbConfig = serde_json::from_str(&config_content)?;
        println!("Loaded configuration from config.json");
        return Ok(config);
    }
    
    // Create default config file
    let default_config = UsbConfig::default();
    let config_content = toml::to_string_pretty(&default_config)?;
    fs::write("config.toml", config_content)?;
    println!("Created default config.toml file");
    
    Err("No configuration file found, using defaults".into())
}

fn monitor_usb_devices(config: UsbConfig, tx: mpsc::Sender<UsbEvent>) {
    loop {
        // Get current USB devices
        let current_devices = get_usb_devices();
        
        // Check for our target device
        let target_device = current_devices.iter()
            .find(|device| {
                device.vendor_id == config.vendor_id && 
                device.device_id == config.device_id
            });

        // Send device state change event
        let event = UsbEvent {
            device_id: config.device_id.clone(),
            vendor_id: config.vendor_id.clone(),
            connected: target_device.is_some(),
        };
        
        if let Err(e) = tx.send(event) {
            eprintln!("Failed to send USB event: {}", e);
        }

        // Wait before next check
        thread::sleep(Duration::from_secs(1));
    }
}

fn get_usb_devices() -> Vec<UsbDevice> {
    let mut devices = Vec::new();
    
    // Use system_profiler on macOS to get USB device information
    let output = Command::new("system_profiler")
        .args(&["SPUSBDataType"])
        .output();

    if let Ok(output) = output {
        let output_str = String::from_utf8_lossy(&output.stdout);
        
        let mut current_vendor_id = String::new();
        let mut current_product_id = String::new();
        
        for line in output_str.lines() {
            let line = line.trim();
            
            if line.contains("Vendor ID:") {
                // Extract vendor ID from line like "Vendor ID: 0x05ac (Apple Inc.)"
                if let Some(start) = line.find("0x") {
                    if let Some(end) = line[start..].find(' ') {
                        current_vendor_id = line[start + 2..start + end].to_lowercase();
                    } else {
                        // If no space after hex, take the rest of the line
                        current_vendor_id = line[start + 2..].to_lowercase();
                    }
                }
            } else if line.contains("Product ID:") {
                // Extract product ID from line like "Product ID: 0x12a8"
                if let Some(start) = line.find("0x") {
                    if let Some(end) = line[start..].find(' ') {
                        current_product_id = line[start + 2..start + end].to_lowercase();
                    } else {
                        // If no space after hex, take the rest of the line
                        current_product_id = line[start + 2..].to_lowercase();
                    }
                }
                
                // If we have both IDs, create a device entry
                if !current_vendor_id.is_empty() && !current_product_id.is_empty() {
                    devices.push(UsbDevice {
                        vendor_id: current_vendor_id.clone(),
                        device_id: current_product_id.clone(),
                        connected: true,
                    });
                    
                    // Reset for next device
                    current_vendor_id.clear();
                    current_product_id.clear();
                }
            }
        }
    }

    devices
}

fn handle_device_changes(rx: mpsc::Receiver<UsbEvent>, device_state: Arc<Mutex<HashMap<String, bool>>>, config: UsbConfig) {
    for event in rx {
        let mut state = device_state.lock().unwrap();
        let device_key = format!("{}:{}", event.vendor_id, event.device_id);
        
        // Check if connection state changed
        let current_connected = event.connected;
        let previous_connected = state.get(&device_key).copied();
        
        if previous_connected != Some(current_connected) {
            // State changed, run appropriate BetterDisplay command
            if current_connected {
                println!("Device connected! Running connect command...");
                run_betterdisplay_command(&config.connect_ddc);
            } else {
                println!("Device disconnected! Running disconnect command...");
                run_betterdisplay_command(&config.disconnect_ddc);
            }
            
            // Update state
            state.insert(device_key, current_connected);
        }
    }
}

fn run_betterdisplay_command(ddc: &str) {
    let output = Command::new("betterdisplaycli")
        .args(&["set", "--ddc", ddc, "--vcp", "inputSelect"])
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                println!("BetterDisplay command executed successfully for DDC: {}", ddc);
            } else {
                eprintln!("BetterDisplay command failed for DDC: {}", ddc);
                if let Ok(stderr) = String::from_utf8(output.stderr) {
                    eprintln!("Error: {}", stderr);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to execute BetterDisplay command: {}", e);
        }
    }
}

#[derive(Clone)]
struct UsbEvent {
    device_id: String,
    vendor_id: String,
    connected: bool,
}