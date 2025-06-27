# Rust RP2350 - Raspberry Pi Pico 2W
Embedded Rust programs for the RP2350 MCU (Raspberry Pi Pico 2 W). Demonstrating practical applications and learning resources.  

## How to run the programs?
- Install / prepare the prerequisites.
- Connect the developer board to your computer.
- Move into the required project.
- Use cargo to run the program - [Run Guide](https://doc.rust-lang.org/book/ch14-01-release-profiles.html)
    - Run command to run in dev profile - cargo run
    - Run command to run in release profile - cargo run --release

## Prerequisites

### Hardware
- The Raspberry Pi Pico 2 W - [Hardware Guide](https://www.raspberrypi.com/documentation/microcontrollers/pico-series.html#pico2w-technical-specification)

### Software Installations
- Install "The Rust Programming Language" - [Installation Guide](https://rust-lang.github.io/rustup/installation/index.html)
- Pico-SDK - [Download Link](https://github.com/raspberrypi/pico-sdk/releases)
- Picotool - [Download Link](https://github.com/raspberrypi/picotool/releases)

### rustup Setup
- Run command - rustup target add thumbv8m.main-none-eabihf
- Run command - rustup target add riscv32imac-unknown-none-elf

### Components Explored
- [x] USB Serial Connection
- [x] ZS-042 RTC Module (a.k.a DS3231)
- [x] SSD1306 0.96 I2C OLED Display
- [x] DHT11 Temperature and Humidity Sensor
- [ ] BMP 280 Pressure Sensor
- [ ] BH1750 Ambient Light Sensor
