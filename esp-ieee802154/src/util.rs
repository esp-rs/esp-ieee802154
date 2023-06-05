use crate::binary::include::{
    ieee802154_coex_event_t, ieee802154_coex_event_t_IEEE802154_IDLE,
    ieee802154_coex_event_t_IEEE802154_LOW, ieee802154_coex_event_t_IEEE802154_MIDDLE,
};

extern "C" {
    fn esp_coex_ieee802154_txrx_pti_set(event: ieee802154_coex_event_t);
}

#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Ieee802154TxrxScene {
    Ieee802154SceneIdle,
    Ieee802154SceneTx,
    Ieee802154SceneRx,
    Ieee802154SceneTxAt,
    Ieee802154SceneRxAt,
}

pub fn ieee802154_set_txrx_pti(txrx_scene: Ieee802154TxrxScene) {
    match txrx_scene {
        Ieee802154TxrxScene::Ieee802154SceneIdle => {
            unsafe { esp_coex_ieee802154_txrx_pti_set(ieee802154_coex_event_t_IEEE802154_IDLE) };
        }
        Ieee802154TxrxScene::Ieee802154SceneTx | Ieee802154TxrxScene::Ieee802154SceneRx => {
            unsafe { esp_coex_ieee802154_txrx_pti_set(ieee802154_coex_event_t_IEEE802154_LOW) };
        }
        Ieee802154TxrxScene::Ieee802154SceneTxAt | Ieee802154TxrxScene::Ieee802154SceneRxAt => {
            unsafe { esp_coex_ieee802154_txrx_pti_set(ieee802154_coex_event_t_IEEE802154_MIDDLE) };
        }
    }
}

pub(crate) fn rssi_to_lqi(rssi: i8) -> u8 {
    if rssi < -80 {
        0
    } else if rssi > -30 {
        0xff
    } else {
        let lqi_convert = ((rssi as u32).wrapping_add(80)) * 255;
        (lqi_convert / 50) as u8
    }
}

pub(crate) fn freq_to_channel(freq: u8) -> u8 {
    (freq - 3) / 5 + 11
}

pub(crate) fn channel_to_freq(channel: u8) -> u8 {
    (channel - 11) * 5 + 3
}
