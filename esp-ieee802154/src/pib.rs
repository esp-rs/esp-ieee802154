use core::{cell::RefCell, default};

use critical_section::Mutex;

use crate::{
    hal::{
        ieee802154_hal_set_cca_mode, ieee802154_hal_set_cca_threshold,
        ieee802154_hal_set_coordinator, ieee802154_hal_set_freq,
        ieee802154_hal_set_multipan_enable_mask, ieee802154_hal_set_multipan_ext_addr,
        ieee802154_hal_set_multipan_panid, ieee802154_hal_set_multipan_short_addr,
        ieee802154_hal_set_pending_mode, ieee802154_hal_set_power, ieee802154_hal_set_promiscuous,
        ieee802154_hal_set_rx_auto_ack, ieee802154_hal_set_tx_auto_ack,
        ieee802154_hal_set_tx_enhance_ack,
    },
    utils::channel_to_freq,
};

const IEEE802154_MULTIPAN_MAX: usize = 4;
const IEEE802154_FRAME_EXT_ADDR_SIZE: usize = 8;
const IEEE802154_MULTIPAN_0: u8 = 0;
const CONFIG_IEEE802154_CCA_THRESHOLD: i8 = 1;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum Ieee802154PendingMode {
    #[default]
    Ieee802154AutoPendingDisable = 0x0,
    Ieee802154AutoPendingEnable = 0x1,
    Ieee802154AutoPendingEnhanced = 0x2,
    Ieee802154AutoPendingZigbee = 0x3,
}

#[derive(Debug, Default, Clone, Copy)]
pub enum Ieee802154CcaMode {
    #[default]
    Ieee802154CcaModeCarrier = 0x00,
    Ieee802154CcaModeEd = 0x01,
    Ieee802154CcaModeCarrierOrEd = 0x02,
    Ieee802154CcaModeCarrierAndEd = 0x03,
}

#[derive(Debug, Default, Clone, Copy)]
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
            promiscuous: true,
            coordinator: false,
            rx_when_idle: false,
            txpower: 10,
            channel: 11,
            pending_mode: Ieee802154PendingMode::Ieee802154AutoPendingDisable,
            multipan_mask: 1 << IEEE802154_MULTIPAN_0,
            panid: [0u16; 4],
            short_addr: [0u16; IEEE802154_MULTIPAN_MAX],
            ext_addr: [[0xffu8; IEEE802154_FRAME_EXT_ADDR_SIZE]; IEEE802154_MULTIPAN_MAX],
            cca_threshold: CONFIG_IEEE802154_CCA_THRESHOLD,
            cca_mode: Ieee802154CcaMode::Ieee802154CcaModeEd,
        });
    });
}

// pub fn ieee802154_pib_set_promiscuous(enable: bool) {
//     if (ieee802154_pib.promiscuous != enable) {
//         ieee802154_pib.promiscuous = enable;
//         set_pending();
//     }
// }

pub fn ieee802154_pib_update() {
    // if (ieee802154_pib_is_pending()) {
    critical_section::with(|cs| {
        let mut pib = PIB.borrow_ref_mut(cs);
        let pib = pib.as_mut().unwrap();

        ieee802154_hal_set_freq(channel_to_freq(pib.channel));
        ieee802154_hal_set_power(ieee802154_txpower_convert(pib.txpower));

        ieee802154_hal_set_multipan_enable_mask(pib.multipan_mask);
        ieee802154_set_multipan_hal(pib.multipan_mask, &pib);

        ieee802154_hal_set_cca_mode(pib.cca_mode);
        ieee802154_hal_set_cca_threshold(pib.cca_threshold);

        ieee802154_hal_set_tx_auto_ack(pib.auto_ack_tx);
        ieee802154_hal_set_rx_auto_ack(pib.auto_ack_rx);
        ieee802154_hal_set_tx_enhance_ack(pib.enhance_ack_tx);

        ieee802154_hal_set_coordinator(pib.coordinator);
        ieee802154_hal_set_promiscuous(pib.promiscuous);
        ieee802154_hal_set_pending_mode(
            pib.pending_mode == Ieee802154PendingMode::Ieee802154AutoPendingEnhanced,
        );
    });

    // clr_pending();
    // }
}

pub fn ieee802154_set_multipan_hal(multipan_mask: u8, pib: &Ieee802154Pib) {
    for index in 0..IEEE802154_MULTIPAN_MAX {
        if (pib.multipan_mask & (1 << index)) != 0 {
            ieee802154_hal_set_multipan_panid(index.into(), pib.panid[index]);
            ieee802154_hal_set_multipan_short_addr(index.into(), pib.short_addr[index]);
            ieee802154_hal_set_multipan_ext_addr(
                index.into(),
                pib.ext_addr[index].as_ptr() as *const u8,
            );
        }
    }
}

const IEEE802154_TXPOWER_VALUE_MAX: i8 = 13;
const IEEE802154_TXPOWER_VALUE_MIN: i8 = -32;

pub fn ieee802154_txpower_convert(txpower: i8) -> u8 {
    let mut ieee820154_txpower_value = 0;
    if txpower > IEEE802154_TXPOWER_VALUE_MAX {
        ieee820154_txpower_value = 15;
    } else if txpower < IEEE802154_TXPOWER_VALUE_MIN {
        ieee820154_txpower_value = 0;
    } else {
        ieee820154_txpower_value = ((txpower - IEEE802154_TXPOWER_VALUE_MIN) / 3) as u8;
    }
    return ieee820154_txpower_value;
}
