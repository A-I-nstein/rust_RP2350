#![no_std]
#![no_main]

use embedded_hal::{delay::DelayNs, i2c::I2c};
use nobcd::BcdNumber;
use panic_halt as _;
use rp235x_hal::{
    block::ImageDef,
    clocks, entry,
    fugit::RateExtU32,
    gpio::{FunctionI2C, Pin, Pins},
    pac::Peripherals,
    usb::UsbBus,
    I2C, Sio, Timer, Watchdog,
};
use usb_device::{class_prelude::*, prelude::*};
use usbd_serial::{embedded_io::Write, SerialPort};

#[unsafe(link_section = ".start_block")]
#[used]
pub static IMAGE_DEF: ImageDef = ImageDef::secure_exe();

const XTAL_FREQ_HZ: u32 = 12_000_000_u32;

const ZS042_ADDR: u8 = 0x68;

#[repr(u8)]
enum ZS042 {
    Seconds,
    Minutes,
    Hours,
    Day,
    Date,
    Month,
    Year,
}

enum DAY {
    Sun = 1,
    Mon = 2,
    Tue = 3,
    Wed = 4,
    Thu = 5,
    Fri = 6,
    Sat = 7,
}

struct DateTime {
    sec: u8,
    min: u8,
    hrs: u8,
    day: u8,
    dat: u8,
    mon: u8,
    yea: u8,
}

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

    let mut rtc_module = I2C::i2c1(
        peripherals.I2C1,
        sda_pin,
        scl_pin,
        400.kHz(),
        &mut peripherals.RESETS,
        &clocks.system_clock,
    );

    let start_dt = DateTime {
        sec: 0,
        min: 0,
        hrs: 0,
        day: DAY::Fri as u8,
        dat: 24,
        mon: 6,
        yea: 25,
    };

    let sec: [u8; 1] = BcdNumber::new(start_dt.sec).unwrap().bcd_bytes();
    rtc_module
        .write(ZS042_ADDR, &[ZS042::Seconds as u8, sec[0]])
        .unwrap();

    let min: [u8; 1] = BcdNumber::new(start_dt.min).unwrap().bcd_bytes();
    rtc_module
        .write(ZS042_ADDR, &[ZS042::Minutes as u8, min[0]])
        .unwrap();

    let hrs: [u8; 1] = BcdNumber::new(start_dt.hrs).unwrap().bcd_bytes();
    rtc_module
        .write(ZS042_ADDR, &[ZS042::Hours as u8, hrs[0]])
        .unwrap();

    let day: [u8; 1] = BcdNumber::new(start_dt.day).unwrap().bcd_bytes();
    rtc_module
        .write(ZS042_ADDR, &[ZS042::Day as u8, day[0]])
        .unwrap();

    let dat: [u8; 1] = BcdNumber::new(start_dt.dat).unwrap().bcd_bytes();
    rtc_module
        .write(ZS042_ADDR, &[ZS042::Date as u8, dat[0]])
        .unwrap();

    let mon: [u8; 1] = BcdNumber::new(start_dt.mon).unwrap().bcd_bytes();
    rtc_module
        .write(ZS042_ADDR, &[ZS042::Month as u8, mon[0]])
        .unwrap();

    let yea: [u8; 1] = BcdNumber::new(start_dt.yea).unwrap().bcd_bytes();
    rtc_module
        .write(ZS042_ADDR, &[ZS042::Year as u8, yea[0]])
        .unwrap();

    timer.delay_ms(100);

    let mut last_send_time_us = timer.get_counter().ticks();

    loop {
        let current_time_us = timer.get_counter().ticks();

        if current_time_us - last_send_time_us >= 1_000_000 {
            last_send_time_us = current_time_us;

            let mut data: [u8; 7] = [0_u8; 7];
            rtc_module.write(ZS042_ADDR, &[0_u8]).unwrap();
            rtc_module.read(ZS042_ADDR, &mut data).unwrap();

            let sec = BcdNumber::from_bcd_bytes([data[0] & 0x7f])
                .unwrap()
                .value::<u8>();
            let min = BcdNumber::from_bcd_bytes([data[1]]).unwrap().value::<u8>();
            let hrs = BcdNumber::from_bcd_bytes([data[2] & 0x3f])
                .unwrap()
                .value::<u8>();
            let dat = BcdNumber::from_bcd_bytes([data[4]]).unwrap().value::<u8>();
            let mon = BcdNumber::from_bcd_bytes([data[5]]).unwrap().value::<u8>();
            let yea = BcdNumber::from_bcd_bytes([data[6]]).unwrap().value::<u8>();
            let day = match BcdNumber::from_bcd_bytes([data[3]]).unwrap().value::<u8>() {
                1 => "Sunday",
                2 => "Monday",
                3 => "Tuesday",
                4 => "Wednesday",
                5 => "Thursday",
                6 => "Friday",
                7 => "Saturday",
                _ => "",
            };

            let mut buffer = [0u8; 32];
            let mut writer = buffer.as_mut_slice();

            write!(
                writer,
                "{}, {}/{}/20{}, {:02}:{:02}:{:02}\r\n",
                day, dat, mon, yea, hrs, min, sec
            )
            .unwrap();
            let _ = serial.write(&buffer[0..]);
        }

        usb_dev.poll(&mut [&mut serial]);
    }
}
