use core::{cell::RefCell, default};

use critical_section::Mutex;

const IEEE802154_MULTIPAN_MAX: usize = 4;
const IEEE802154_FRAME_EXT_ADDR_SIZE: usize = 8;
const IEEE802154_MULTIPAN_0: u8 = 0;
const CONFIG_IEEE802154_CCA_THRESHOLD: i8 = 1;

#[derive(Debug, Default)]
pub enum Ieee802154PendingMode {
    #[default]
    Ieee802154AutoPendingDisable = 0x0,
    Ieee802154AutoPendingEnable = 0x1,
    Ieee802154AutoPendingEnhanced = 0x2,
    Ieee802154AutoPendingZigbee = 0x3,
}

#[derive(Debug, Default)]
pub enum Ieee802154CcaMode {
    #[default]
    Ieee802154CcaModeCarrier = 0x00,
    Ieee802154CcaModeEd = 0x01,
    Ieee802154CcaModeCarrierOrEd = 0x02,
    Ieee802154CcaModeCarrierAndEd = 0x03,
}

#[derive(Debug, Default)]
pub struct Ieee802154Pib {
    auto_ack_tx: bool,
    auto_ack_rx: bool,
    enhance_ack_tx: bool,
    promiscuous: bool,
    coordinator: bool,
    rx_when_idle: bool,
    txpower: i8,
    channel: u8,
    pending_mode: Ieee802154PendingMode,
    multipan_mask: u8,
    panid: [u16; IEEE802154_MULTIPAN_MAX],
    short_addr: [u16; IEEE802154_MULTIPAN_MAX],
    ext_addr: [[u8; IEEE802154_FRAME_EXT_ADDR_SIZE]; IEEE802154_MULTIPAN_MAX],
    cca_threshold: i8,
    cca_mode: Ieee802154CcaMode,
}

static PIB: Mutex<RefCell<Option<Ieee802154Pib>>> = Mutex::new(RefCell::new(None));

pub fn ieee802154_pib_init() {
    critical_section::with(|cs| {
        PIB.borrow_ref_mut(cs).replace(Ieee802154Pib {
            auto_ack_tx: true,
            auto_ack_rx: true,
            enhance_ack_tx: false,
            promiscuous: false,
            coordinator: false,
            rx_when_idle: false,
            txpower: 10,
            channel: 11,
            pending_mode: Ieee802154PendingMode::Ieee802154AutoPendingDisable,
            multipan_mask: 1 << IEEE802154_MULTIPAN_0,
            panid: [0u16; 4],
            short_addr: [0u16; IEEE802154_MULTIPAN_MAX],
            ext_addr: [[0u8; IEEE802154_FRAME_EXT_ADDR_SIZE]; IEEE802154_MULTIPAN_MAX],
            cca_threshold: CONFIG_IEEE802154_CCA_THRESHOLD,
            cca_mode: Ieee802154CcaMode::Ieee802154CcaModeEd,
        });
    });
}
