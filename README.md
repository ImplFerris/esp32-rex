# esp32-rex
Dinosaur Game written in Rust for the ESP32 with an OLED display, using the Embassy framework. You can also find the Pico2 version of this code [here](https://github.com/ImplFerris/pico-rex).

## Hardware Requirements
- ESP32 (WROOM Dev Kit 1)
- SSD1306 OLED I2C 128x64 Display
- Push button (with a cap) 
- Jumper wires and breadboard

## Circuit

| Pico Pin | Component               |
|----------|-------------------------|
| GPIO 21  | SDA pin of OLED         |
| GPIO 22  | SCL pin of OLED         |
| 3.3V     | VCC pin of OLED         |
| GND      | GND pin of OLED         |
| GPIO 4   | One side of push button |
| GND      | Other side of push button |
 

## TODO
1. Implement running illusion for the T-Rex
2. Display start menu
3. Smooth gaming experience!

