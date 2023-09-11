#![no_std]
#![no_main]

#[cfg(feature = "esp32c6")]
use esp32c6_hal as esp_hal;
#[cfg(feature = "esp32h2")]
use esp32h2_hal as esp_hal;
use esp_backtrace as _;
use esp_hal::{
    clock::{ClockControl, CpuClock},
    peripherals::Peripherals,
    prelude::*,
};
use esp_ieee802154::*;
use esp_println::println;

#[entry]
fn main() -> ! {
    esp_println::logger::init_logger(log::LevelFilter::Info);

    let peripherals = Peripherals::take();
    let mut system = peripherals.PCR.split();
    #[cfg(feature = "esp32c6")]
    let _clocks = ClockControl::configure(system.clock_control, CpuClock::Clock160MHz).freeze();
    #[cfg(feature = "esp32h2")]
    let _clocks = ClockControl::configure(system.clock_control, CpuClock::Clock96MHz).freeze();

    println!("Start");

    let (.., radio) = peripherals.RADIO.split();
    let mut ieee802154 = Ieee802154::new(radio, &mut system.radio_clock_control);

    ieee802154.set_config(Config {
        channel: 15,
        promiscuous: false,
        rx_when_idle: true,
        auto_ack_rx: true,
        auto_ack_tx: true,
        pan_id: Some(0x4242),
        short_addr: Some(0x2323),
        ..Config::default()
    });

    println!("start receiving");
    ieee802154.start_receive();

    loop {
        if let Some(frame) = ieee802154.get_received() {
            println!("Received {:?}\n", &frame);
        }
    }
}
