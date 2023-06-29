//! Low-level [IEEE 802.15.4] driver for the ESP32-C6 and ESP32-H2
//!
//! Implements the PHY/MAC layers of the IEEE 802.15.4 protocol stack, and
//! supports sending and receiving of raw frames.
//!
//! [IEEE 802.15.4]: https://en.wikipedia.org/wiki/IEEE_802.15.4

#![no_std]
#![feature(c_variadic)]

use byte::{BytesExt, TryRead};
#[cfg(feature = "esp32c6")]
use esp32c6_hal as esp_hal;
#[cfg(feature = "esp32h2")]
use esp32h2_hal as esp_hal;
use esp_hal::system::RadioClockControl;
use heapless::Vec;
use ieee802154::mac::{self, FooterMode, FrameSerDesContext};

use self::{
    frame::FRAME_SIZE,
    pib::{CONFIG_IEEE802154_CCA_THRESHOLD, IEEE802154_FRAME_EXT_ADDR_SIZE},
    raw::*,
};
pub use self::{
    frame::{Frame, ReceivedFrame},
    pib::{CcaMode, PendingMode},
    raw::RawReceived,
};

mod binary;
mod compat;
mod frame;
mod hal;
mod pib;
mod raw;

#[no_mangle]
extern "C" fn rtc_clk_xtal_freq_get() -> i32 {
    0
}

/// IEEE 802.15.4 errors
#[derive(Debug, Clone, Copy)]
pub enum Error {
    /// The requested data is bigger than available range, and/or the offset is
    /// invalid
    Incomplete,
    /// The requested data content is invalid
    BadInput,
}

/// IEEE 802.15.4 driver configuration
#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub auto_ack_tx: bool,
    pub auto_ack_rx: bool,
    pub enhance_ack_tx: bool,
    pub promiscuous: bool,
    pub coordinator: bool,
    pub rx_when_idle: bool,
    pub txpower: i8,
    pub channel: u8,
    pub cca_threshold: i8,
    pub cca_mode: CcaMode,
    pub pan_id: Option<u16>,
    pub short_addr: Option<u16>,
    pub ext_addr: Option<u64>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            auto_ack_tx: Default::default(),
            auto_ack_rx: Default::default(),
            enhance_ack_tx: Default::default(),
            promiscuous: Default::default(),
            coordinator: Default::default(),
            rx_when_idle: Default::default(),
            txpower: 10,
            channel: 15,
            cca_threshold: CONFIG_IEEE802154_CCA_THRESHOLD,
            cca_mode: CcaMode::Ed,
            pan_id: None,
            short_addr: None,
            ext_addr: None,
        }
    }
}

/// IEEE 802.15.4 driver
#[derive(Debug, Clone, Copy)]
pub struct Ieee802154 {
    _private: (),
    _align: u32,
    transmit_buffer: [u8; FRAME_SIZE],
}

impl Ieee802154 {
    /// Construct a new driver, enabling the IEEE 802.15.4 radio in the process
    pub fn new(radio_clocks: &mut RadioClockControl) -> Self {
        esp_ieee802154_enable(radio_clocks);

        Self {
            _private: (),
            _align: 0,
            transmit_buffer: [0u8; FRAME_SIZE],
        }
    }

    /// Set the configuration for the driver
    pub fn set_config(&mut self, cfg: Config) {
        set_auto_ack_tx(cfg.auto_ack_tx);
        set_auto_ack_rx(cfg.auto_ack_rx);
        set_enhance_ack_tx(cfg.enhance_ack_tx);
        set_promiscuous(cfg.promiscuous);
        set_coordinator(cfg.coordinator);
        set_rx_when_idle(cfg.rx_when_idle);
        set_tx_power(cfg.txpower);
        set_channel(cfg.channel);
        set_cca_theshold(cfg.cca_threshold);
        set_cca_mode(cfg.cca_mode);

        if let Some(pan_id) = cfg.pan_id {
            set_panid(0, pan_id);
        }

        if let Some(short_addr) = cfg.short_addr {
            set_short_address(0, short_addr);
        }

        if let Some(ext_addr) = cfg.ext_addr {
            let mut address = [0u8; IEEE802154_FRAME_EXT_ADDR_SIZE];
            address.copy_from_slice(&ext_addr.to_be_bytes()); // LE or BE?

            set_extended_address(0, address);
        }
    }

    /// Start receiving frames
    pub fn start_receive(&mut self) {
        ieee802154_receive();
    }

    /// Return the raw data of a received frame
    pub fn get_raw_received(&mut self) -> Option<RawReceived> {
        ieee802154_poll()
    }

    /// Get a received frame, if available
    pub fn get_received(&mut self) -> Option<Result<ReceivedFrame, Error>> {
        let poll_res = ieee802154_poll();
        if let Some(raw) = poll_res {
            let decode_res =
                mac::Frame::try_read(&raw.data[1..][..raw.data[0] as usize], FooterMode::Explicit);

            if let Ok((decoded, _)) = decode_res {
                let rssi = raw.data[raw.data[0] as usize - 1] as i8; // crc is not written to rx buffer

                Some(Ok(ReceivedFrame {
                    frame: Frame {
                        header: decoded.header,
                        content: decoded.content,
                        payload: Vec::from_slice(decoded.payload).unwrap(),
                        footer: decoded.footer,
                    },
                    channel: raw.channel,
                    rssi,
                    lqi: rssi_to_lqi(rssi),
                }))
            } else {
                Some(Err(match decode_res.err().unwrap() {
                    byte::Error::Incomplete | byte::Error::BadOffset(_) => Error::Incomplete,
                    byte::Error::BadInput { .. } => Error::BadInput,
                }))
            }
        } else {
            None
        }
    }

    /// Transmit a frame
    pub fn transmit(&mut self, frame: &Frame) -> Result<(), Error> {
        let frm = mac::Frame {
            header: frame.header,
            content: frame.content,
            payload: &frame.payload,
            footer: frame.footer,
        };

        let mut offset = 1usize;
        self.transmit_buffer
            .write_with(
                &mut offset,
                frm,
                &mut FrameSerDesContext::no_security(FooterMode::Explicit),
            )
            .unwrap();
        self.transmit_buffer[0] = (offset - 1) as u8;

        ieee802154_transmit(self.transmit_buffer.as_ptr() as *const u8, false); // what about CCA?

        Ok(())
    }
}

fn rssi_to_lqi(rssi: i8) -> u8 {
    if rssi < -80 {
        0
    } else if rssi > -30 {
        0xff
    } else {
        let lqi_convert = ((rssi as u32).wrapping_add(80)) * 255;
        (lqi_convert / 50) as u8
    }
}
