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
    Delay,
};
use esp_ieee802154::*;
use esp_println::println;
use ieee802154::mac::{Header, PanId, ShortAddress};

#[entry]
fn main() -> ! {
    esp_println::logger::init_logger(log::LevelFilter::Info);

    let peripherals = Peripherals::take();
    let mut system = peripherals.PCR.split();
    #[cfg(feature = "esp32c6")]
    let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock160MHz).freeze();
    #[cfg(feature = "esp32h2")]
    let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock96MHz).freeze();

    let mut delay = Delay::new(&clocks);

    println!("Start");

    let (.., radio) = peripherals.RADIO.split();
    let mut ieee802154 = Ieee802154::new(radio, &mut system.radio_clock_control);

    ieee802154.set_config(Config {
        channel: 15,
        promiscuous: false,
        pan_id: Some(0x4242),
        short_addr: Some(0x2222),
        ..Config::default()
    });

    let mut seq_number = 0u8;

    loop {
        ieee802154
            .transmit(&Frame {
                header: Header {
                    frame_type: ieee802154::mac::FrameType::Data,
                    frame_pending: false,
                    ack_request: true,
                    pan_id_compress: false,
                    seq_no_suppress: false,
                    ie_present: false,
                    version: ieee802154::mac::FrameVersion::Ieee802154_2003,
                    seq: seq_number,
                    destination: Some(ieee802154::mac::Address::Short(
                        PanId(0x4242),
                        ShortAddress(0x2323),
                    )),
                    source: None,
                    auxiliary_security_header: None,
                },
                content: ieee802154::mac::FrameContent::Data,
                payload: heapless::Vec::from_slice(b"Hello World").unwrap(),
                footer: [0u8; 2],
            })
            .ok();

        println!("Send frame with sequence number {seq_number}");
        delay.delay_ms(1000u32);
        seq_number = seq_number.wrapping_add(1);
    }
}
