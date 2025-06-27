#![no_std]
#![no_main]

use embedded_graphics::{
    image::Image, mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder}, pixelcolor::BinaryColor, prelude::Point, text::{Baseline, Text}, Drawable
};
use embedded_hal::delay::DelayNs;
use panic_halt as _;
use rp235x_hal::{
    block::ImageDef,
    clocks, entry,
    fugit::RateExtU32,
    gpio::{FunctionI2C, Pin, Pins},
    pac::Peripherals,
    Sio, Timer, Watchdog, I2C,
};
use ssd1306::{
    mode::DisplayConfig, prelude::DisplayRotation, size::DisplaySize128x64, I2CDisplayInterface,
    Ssd1306,
};
use tinybmp::Bmp;

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

    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate180)
        .into_buffered_graphics_mode();
    display.init().unwrap();
    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    Text::with_baseline(
        "Hello, World!",
        Point::new(0, 0),
        text_style,
        Baseline::Top,
    )
    .draw(&mut display)
    .unwrap();

    let bmp = Bmp::from_slice(include_bytes!("../ferris.bmp")).unwrap();
    let img = Image::new(&bmp, Point::new(0, 10));
    img.draw(&mut display).unwrap();
    display.flush().unwrap();

    loop {
        timer.delay_ms(500);
    }
}
