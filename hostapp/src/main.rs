use serde::Deserialize;
use std::fs;
use std::io::{self, Read};
use std::process::Command;
use std::thread;
use std::time::Duration;

const TOGGLE_COMMAND: &str = "TOGGLE_INPUT"; // String command to listen for

#[derive(Deserialize)]
struct Config {
  port: String,
  baud_rate: u32,
}

fn load_config() -> io::Result<Config> {
  let mut config_path = dirs::home_dir().expect("Failed to determine home directory");
  config_path.push(".config");
  config_path.push("input-switcher");
  config_path.push("config.toml");

  println!("Loading configuration from: {:?}", config_path);

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
  let port_name = config.port.clone();
  let baud_rate = config.baud_rate;

  let mut current_input = 0;

  println!("Using port: {} with baud rate: {}", port_name, baud_rate);

  loop {
    println!("Attempting to connect to the serial port...");
    if let Some(mut port) = open_serial_port(&port_name, baud_rate) {
      println!("Connected to the serial port. Listening for '{}' command...", TOGGLE_COMMAND);

      let mut buffer = vec![0; 1024];
      let mut received_data = Vec::new();

      loop {
        match port.read(buffer.as_mut_slice()) {
          Ok(bytes_read) => {
            received_data.extend_from_slice(&buffer[..bytes_read]);

            // Convert received bytes to string and check if it contains the toggle command
            if let Ok(received_string) = String::from_utf8(received_data.clone()) {
              if received_string.contains(TOGGLE_COMMAND) {
                println!("Toggle command detected! Triggering system command...");

                current_input = (current_input + 1) % 2;

                println!("Setting input to {}", current_input);
                let input_name = if current_input == 0 { "15" } else { "18" };

                // set the input to macbook
                Command::new("betterdisplaycli")
                  .arg("set")
                  .arg(format!("--ddc={}", input_name))
                  .arg("--vcp=inputSelect")
                  .spawn()
                  .expect("Failed to execute command");

                // Clear the buffer after detecting the command
                received_data.clear();
              }
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
