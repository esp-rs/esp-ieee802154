#![no_std]
#![no_main]

use embedded_hal_nb::serial::Read;
use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl, peripherals::Peripherals, prelude::*, reset::software_reset,
    system::SystemControl, uart::Uart,
};
use esp_ieee802154::*;
use esp_println::println;

#[entry]
fn main() -> ! {
    let mut peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::max(system.clock_control).freeze();

    let mut uart0 = Uart::new(peripherals.UART0, &clocks);
    let mut cnt = 0;
    let mut read = [0u8; 2];
    loop {
        if let nb::Result::Ok(c) = uart0.read() {
            if c == b'r' {
                continue;
            }

            read[cnt] = c;
            cnt += 1;

            if cnt >= 2 {
                break;
            }
        }
    }

    let channel: u8 = unsafe { core::str::from_utf8_unchecked(&read) }
        .parse()
        .unwrap();

    let mut ieee802154 = Ieee802154::new(peripherals.IEEE802154, &mut peripherals.RADIO_CLK);

    ieee802154.set_config(Config {
        channel,
        promiscuous: true,
        rx_when_idle: true,
        auto_ack_rx: false,
        auto_ack_tx: false,
        ..Config::default()
    });

    ieee802154.start_receive();

    loop {
        if let Some(frame) = ieee802154.get_raw_received() {
            println!("@RAW {:02x?}", &frame.data);
        }

        if let nb::Result::Ok(c) = uart0.read() {
            if c == b'r' {
                software_reset();
            }
        }
    }
}
