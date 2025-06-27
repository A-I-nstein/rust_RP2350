#![no_std]
#![no_main]

use dht_sensor::{dht11, DhtReading};
use embedded_hal::{delay::DelayNs, digital::OutputPin};
use panic_halt as _;
use rp235x_hal::{
    block::ImageDef,
    clocks, entry,
    gpio::{InOutPin, Pins},
    pac::Peripherals,
    usb::UsbBus,
    Sio, Timer, Watchdog,
};
use usb_device::{class_prelude::*, prelude::*};
use usbd_serial::{embedded_io::Write, SerialPort};

#[link_section = ".start_block"]
#[used]
pub static IMAGE_DEF: ImageDef = ImageDef::secure_exe();

const XTAL_FREQ_HZ: u32 = 12_000_000_u32;

#[entry]
fn main() -> ! {
    let mut peripherals = Peripherals::take().unwrap();

    let mut watchdog = Watchdog::new(peripherals.WATCHDOG);

    let clocks = clocks::init_clocks_and_plls(
        XTAL_FREQ_HZ,
        peripherals.XOSC,
        peripherals.CLOCKS,
        peripherals.PLL_SYS,
        peripherals.PLL_USB,
        &mut peripherals.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut timer = Timer::new_timer0(peripherals.TIMER0, &mut peripherals.RESETS, &clocks);

    let usb_bus = UsbBusAllocator::new(UsbBus::new(
        peripherals.USB,
        peripherals.USB_DPRAM,
        clocks.usb_clock,
        true,
        &mut peripherals.RESETS,
    ));

    let mut serial = SerialPort::new(&usb_bus);

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .strings(&[StringDescriptors::default()
            .manufacturer("implRust")
            .product("Ferris")
            .serial_number("TEST")])
        .unwrap()
        .device_class(2)
        .build();

    let sio = Sio::new(peripherals.SIO);
    let pins = Pins::new(
        peripherals.IO_BANK0,
        peripherals.PADS_BANK0,
        sio.gpio_bank0,
        &mut peripherals.RESETS,
    );

    let mut data_pin = InOutPin::new(pins.gpio28);
    let _ = data_pin.set_high();

    timer.delay_ms(100);

    let mut last_send_time_us = timer.get_counter().ticks();

    loop {
        let current_time_us = timer.get_counter().ticks();

        if current_time_us - last_send_time_us >= 1_000_000 {
            last_send_time_us = current_time_us;

            let temp;
            let humi;

            let mut buffer = [0u8; 32];
            let mut writer = buffer.as_mut_slice();

            match dht11::Reading::read(&mut timer, &mut data_pin) {
                Ok(meas) => {
                    temp = meas.temperature;
                    humi = meas.relative_humidity;

                    write!(writer, "Temp: {}, Hum:{}\r\n", temp, humi).unwrap();
                    let _ = serial.write(&buffer[0..]);
                }
                Err(e) => {
                    write!(writer, "{:?}\r\n", e).unwrap();
                    let _ = serial.write(&buffer[0..]);
                }
            };
        }

        usb_dev.poll(&mut [&mut serial]);
    }
}
