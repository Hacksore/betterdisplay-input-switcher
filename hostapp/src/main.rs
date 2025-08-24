use serde::Deserialize;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;

const MAGIC_SEQUENCE: &[u8] = &[0xDE, 0xAD, 0xBE, 0xEF]; // Replace with your magic byte sequence

#[derive(Deserialize)]
struct Config {
    serial: SerialConfig,
}

#[derive(Deserialize)]
struct SerialConfig {
    port: String,
    baud_rate: u32,
}

fn load_config() -> io::Result<Config> {
    let mut config_path = dirs::config_dir().expect("Failed to determine config directory");
    config_path.push("input-switcher/config.toml");

    let config_content = fs::read_to_string(config_path)?;
    let config: Config = toml::from_str(&config_content).expect("Failed to parse config.toml");
    Ok(config)
}

fn open_serial_port(port_name: &str, baud_rate: u32) -> Option<Box<dyn serialport::SerialPort>> {
    match serialport::new(port_name, baud_rate)
        .timeout(Duration::from_millis(100))
        .open()
    {
        Ok(port) => Some(port),
        Err(e) => {
            eprintln!("Failed to open serial port {}: {}", port_name, e);
            None
        }
    }
}

fn main() -> io::Result<()> {
    // Load configuration
    let config = load_config()?;
    let port_name = config.serial.port.clone();
    let baud_rate = config.serial.baud_rate;

    println!("Using port: {} with baud rate: {}", port_name, baud_rate);

    loop {
        println!("Attempting to connect to the serial port...");
        if let Some(mut port) = open_serial_port(&port_name, baud_rate) {
            println!("Connected to the serial port. Listening for magic sequence...");

            let mut buffer = vec![0; 1024];
            let mut received_data = Vec::new();

            loop {
                match port.read(buffer.as_mut_slice()) {
                    Ok(bytes_read) => {
                        received_data.extend_from_slice(&buffer[..bytes_read]);

                        // Check if the magic sequence is in the received data
                        if received_data.windows(MAGIC_SEQUENCE.len()).any(|window| window == MAGIC_SEQUENCE) {
                            println!("Magic sequence detected! Triggering system command...");

                            // Trigger a system command (example: open a file or application)
                            Command::new("open")
                                .arg("/Applications/Calculator.app") // Replace with your desired command
                                .spawn()
                                .expect("Failed to execute command");

                            // Clear the buffer after detecting the sequence
                            received_data.clear();
                        }
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {
                        // Ignore timeout errors
                    }
                    Err(e) => {
                        eprintln!("Error reading from serial port: {}", e);
                        break; // Exit the inner loop to attempt reconnection
                    }
                }
            }
        }

        println!("Reconnecting in 5 seconds...");
        thread::sleep(Duration::from_secs(5));
    }
}
