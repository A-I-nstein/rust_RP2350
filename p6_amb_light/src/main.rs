#![no_std]
#![no_main]

use bh1750_ehal::{Address, BH1750, ContinuesMeasurement};
use embedded_hal::delay::DelayNs;
use panic_halt as _;
use rp235x_hal::{
    block::ImageDef,
    clocks, entry,
    fugit::RateExtU32,
    gpio::{FunctionI2C, Pin, Pins},
    pac::Peripherals,
    usb::UsbBus,
    Sio, Timer, Watchdog, I2C,
};
use usb_device::{class_prelude::*, prelude::*};
use usbd_serial::{embedded_io::Write, SerialPort};

#[unsafe(link_section = ".start_block")]
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

    let sda_pin: Pin<_, FunctionI2C, _> = pins.gpio14.reconfigure();
    let scl_pin: Pin<_, FunctionI2C, _> = pins.gpio15.reconfigure();

    let i2c = I2C::i2c1(
        peripherals.I2C1,
        sda_pin,
        scl_pin,
        400.kHz(),
        &mut peripherals.RESETS,
        &clocks.system_clock,
    );

    let mut bh1750 = BH1750::new(i2c, timer, Address::ADDR_L).unwrap();

    timer.delay_ms(100);

    let mut last_send_time_us = timer.get_counter().ticks();

    loop {
        let current_time_us = timer.get_counter().ticks();

        if current_time_us - last_send_time_us >= 1_000_000 {
            last_send_time_us = current_time_us;

            let measurements = bh1750.get_measurement(ContinuesMeasurement::LOW_RES);

            let mut buffer = [0u8; 32];
            let mut writer = buffer.as_mut_slice();

            write!(
                writer,
                "Lux: {:.1}\r\n",
                measurements
            )
            .unwrap();
            let _ = serial.write(&buffer[0..]);
        }

        usb_dev.poll(&mut [&mut serial]);
    }
}
