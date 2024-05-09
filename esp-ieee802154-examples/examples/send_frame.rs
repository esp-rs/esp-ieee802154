#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl, delay::Delay, peripherals::Peripherals, prelude::*, system::SystemControl,
};
use esp_ieee802154::*;
use esp_println::println;
use ieee802154::mac::{Header, PanId, ShortAddress};

#[entry]
fn main() -> ! {
    let mut peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::max(system.clock_control).freeze();

    let mut ieee802154 = Ieee802154::new(peripherals.IEEE802154, &mut peripherals.RADIO_CLK);

    ieee802154.set_config(Config {
        channel: 15,
        promiscuous: false,
        pan_id: Some(0x4242),
        short_addr: Some(0x2222),
        ..Config::default()
    });

    let delay = Delay::new(&clocks);

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
        delay.delay_millis(1000u32);
        seq_number = seq_number.wrapping_add(1);
    }
}
