use core::cell::RefCell;

use critical_section::Mutex;

use crate::{
    binary::include::{
        ieee802154_coex_event_t_IEEE802154_ACK, ieee802154_coex_event_t_IEEE802154_IDLE_RX,
        ieee802154_coex_event_t_IEEE802154_NORMAL, ieee802154_coex_event_t_IEEE802154_RX_AT,
        ieee802154_coex_event_t_IEEE802154_TX_AT,
    },
    hal::{ieee802154_hal_set_ack_pti, ieee802154_hal_set_normal_pti},
};

extern "C" {
    fn esp_ieee802154_coex_pti_set(event: u32); // TO DO: esp_ieee802154_coex_pti_set --> esp_coex_ieee802154_pti_set
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Ieee802154TxrxScene {
    Ieee802154SceneIdle,
    Ieee802154SceneTx,
    Ieee802154SceneRx,
    Ieee802154SceneTxAt,
    Ieee802154SceneRxAt,
}

static TEST_MODE: Mutex<RefCell<bool>> = Mutex::new(RefCell::new(false));
static TX_PTI: Mutex<RefCell<u8>> = Mutex::new(RefCell::new(0u8));
static RX_PTI: Mutex<RefCell<u8>> = Mutex::new(RefCell::new(0u8));
static ACK_PTI: Mutex<RefCell<u8>> = Mutex::new(RefCell::new(0u8));

pub fn set_test_mode(enable: bool) {
    critical_section::with(|cs| {
        *(TEST_MODE.borrow_ref_mut(cs)) = enable;
    });
}

pub fn get_test_mode() -> bool {
    critical_section::with(|cs| *(TEST_MODE.borrow_ref(cs)))
}

pub fn set_ack_pti() {
    unsafe {
        esp_ieee802154_coex_pti_set(ieee802154_coex_event_t_IEEE802154_ACK); // from libcoexist.a
    }
}

pub fn ieee802154_set_txrx_pti(txrx_scene: Ieee802154TxrxScene) {
    let (s_tx_pti, s_rx_pti, s_ack_pti) = critical_section::with(|cs| {
        (
            *TX_PTI.borrow(cs).borrow(),
            *RX_PTI.borrow(cs).borrow(),
            *ACK_PTI.borrow(cs).borrow(),
        )
    });

    if get_test_mode() {
        if txrx_scene == Ieee802154TxrxScene::Ieee802154SceneTx
            || txrx_scene == Ieee802154TxrxScene::Ieee802154SceneTxAt
        {
            ieee802154_hal_set_normal_pti(s_tx_pti);
        } else if txrx_scene == Ieee802154TxrxScene::Ieee802154SceneRx
            || txrx_scene == Ieee802154TxrxScene::Ieee802154SceneRxAt
        {
            ieee802154_hal_set_normal_pti(s_rx_pti);
        }
        ieee802154_hal_set_ack_pti(s_ack_pti);
    } else {
        match txrx_scene {
            Ieee802154TxrxScene::Ieee802154SceneIdle => {
                unsafe {
                    esp_ieee802154_coex_pti_set(ieee802154_coex_event_t_IEEE802154_IDLE_RX);
                    // from libcoexist.a
                }
            }
            Ieee802154TxrxScene::Ieee802154SceneTx | Ieee802154TxrxScene::Ieee802154SceneRx => {
                unsafe {
                    esp_ieee802154_coex_pti_set(ieee802154_coex_event_t_IEEE802154_NORMAL);
                    // from libcoexist.a
                }
            }
            Ieee802154TxrxScene::Ieee802154SceneTxAt => {
                unsafe {
                    esp_ieee802154_coex_pti_set(ieee802154_coex_event_t_IEEE802154_TX_AT);
                    // from libcoexist.a
                }
            }
            Ieee802154TxrxScene::Ieee802154SceneRxAt => {
                unsafe {
                    esp_ieee802154_coex_pti_set(ieee802154_coex_event_t_IEEE802154_RX_AT);
                    // from libcoexist.a
                }
            }
        }
    }
}
