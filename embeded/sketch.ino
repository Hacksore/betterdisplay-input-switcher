/*
 * Pro Micro Button Serial Monitor
 * 
 * This sketch listens for a button press on pin 2 and sends data over USB serial
 * when the button is pressed. It includes debouncing to prevent false triggers.
 * 
 * Hardware:
 * - Pro Micro with Atmega32U4
 * - Button connected between pin 2 and GND
 * - Optional: 10kΩ pull-up resistor between pin 2 and VCC
 * 
 * Pin Connections:
 * - Button: Pin 2 to GND
 * - Pull-up: Pin 2 to VCC (optional, internal pull-up can be used)
 */

// Button configuration
const int BUTTON_PIN = 2;        // Button connected to pin 2
const unsigned long DEBOUNCE_DELAY = 50;  // Debounce delay in milliseconds

// Button state variables
int buttonState = HIGH;          // Current button state
int lastButtonState = HIGH;      // Previous button state
unsigned long lastDebounceTime = 0;  // Last time button state changed

// Serial message configuration
const char* BUTTON_PRESSED_MSG = "TOGGLE_INPUT";

void setup() {
  // Initialize serial communication
  Serial.begin(9600);
  
  // Wait for serial connection to be established
  while (!Serial) {
    delay(10);
  }
  
  // Initialize button pin with internal pull-up resistor
  pinMode(BUTTON_PIN, INPUT_PULLUP);
  
  // Send startup message
  Serial.println("Pro Micro Button Monitor Ready");
  Serial.println("Waiting for button press...");
}

void loop() {
  // Read the button state
  int reading = digitalRead(BUTTON_PIN);
  
  // Check if button state has changed
  if (reading != lastButtonState) {
    // Reset the debouncing timer
    lastDebounceTime = millis();
  }
  
  // Check if enough time has passed since the last button state change
  if ((millis() - lastDebounceTime) > DEBOUNCE_DELAY) {
    // If the button state has changed
    if (reading != buttonState) {
      buttonState = reading;
      
      // Send appropriate message based on button state
      if (buttonState == LOW) {  // Button pressed (LOW due to pull-up)
        Serial.println(BUTTON_PRESSED_MSG);
        Serial.println("Button was pressed!");
      }
    }
  }
  
  // Save the current button state for next comparison
  lastButtonState = reading;
  
  // Small delay to prevent excessive CPU usage
  delay(10);
}
