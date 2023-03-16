#![no_std]
#![feature(c_variadic)]

use core::cell::RefCell;

use critical_section::Mutex;
use hal::disable_events;
use pib::*;
use util::{get_test_mode, ieee802154_set_txrx_pti, set_ack_pti, Ieee802154TxrxScene};
use utils::ieee802154;

use self::{
    binary::include::{esp_phy_calibration_mode_t_PHY_RF_CAL_FULL, register_chipv7_phy},
    hal::*,
};

mod binary;
mod compat;
mod hal;
mod pib;
#[cfg_attr(feature = "esp32h4", path = "ral/esp32h4.rs")]
#[cfg_attr(feature = "esp32c6", path = "ral/esp32c6.rs")]
mod ral;
mod util;
mod utils;

const PHY_ENABLE_VERSION_PRINT: u32 = 1;

extern "C" {
    pub fn bt_bb_v2_init_cmplx(print_version: u32); // from libbtbb.a

    pub fn bt_bb_set_zb_tx_on_delay(time: u16); // from libbtbb.a

    pub fn phy_version_print(); // from libphy.a
}

static mut RX_BUFFER: [u8; 129] = [0u8; 129]; // just for testing

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Ieee802154State {
    Idle,
    Receive,
    Transmit,
}

static STATE: Mutex<RefCell<Ieee802154State>> = Mutex::new(RefCell::new(Ieee802154State::Idle));

/// Enable the IEEE802.15.4 radio
pub fn esp_ieee802154_enable() {
    ieee802154_enable();
    esp_phy_enable();
    ieee802154_mac_init();

    log::info!("date={:x}", ieee802154().mac_date.read().bits());
}

/// Enable the PHY
fn esp_phy_enable() {
    unsafe {
        let mut calibration_data = binary::include::esp_phy_calibration_data_t {
            version: [0u8; 4],
            mac: [0u8; 6],
            opaque: [0u8; 1894],
        };
        register_chipv7_phy(
            core::ptr::null(),
            &mut calibration_data as *mut binary::include::esp_phy_calibration_data_t,
            esp_phy_calibration_mode_t_PHY_RF_CAL_FULL,
        );
        bt_bb_v2_init_cmplx(PHY_ENABLE_VERSION_PRINT); // bt_bb_v2_init_cmplx // bt_bb_v2_init_cmplx(int print_version);
        phy_version_print(); // libphy.a // extern void phy_version_print(void);
    }
}

/// Enable the IEEE802.15.4 clock and modem
fn ieee802154_enable() {
    #[cfg(not(feature = "esp32c6"))]
    compile_error!("Unsupported target");

    const MODEM_SYSCON: u32 = 0x600A9800;
    const MODEM_LPCON: u32 = 0x600AF000;
    let syscon_clk_conf = (MODEM_SYSCON + 4) as *mut u32;
    let syscon_clk_conf1 = (MODEM_SYSCON + 20) as *mut u32;
    let lpcon_clk_conf = (MODEM_LPCON + 24) as *mut u32;
    unsafe {
        syscon_clk_conf.write_volatile(syscon_clk_conf.read_volatile() | 1 << 24); // hw->clk_conf.clk_zb_mac_en = en;
        syscon_clk_conf1.write_volatile(syscon_clk_conf1.read_volatile() | 1 << 16); // hw->clk_conf1.clk_fe_apb_en = en;
        syscon_clk_conf1.write_volatile(syscon_clk_conf1.read_volatile() | 1 << 15); // hw->clk_conf1.clk_fe_cal_160m_en = en;
        syscon_clk_conf1.write_volatile(syscon_clk_conf1.read_volatile() | 1 << 14); // hw->clk_conf1.clk_fe_160m_en = en;
        syscon_clk_conf1.write_volatile(syscon_clk_conf1.read_volatile() | 1 << 13); // hw->clk_conf1.clk_fe_80m_en = en;
        syscon_clk_conf1.write_volatile(syscon_clk_conf1.read_volatile() | 1 << 17); // hw->clk_conf1.clk_bt_apb_en = en;
        syscon_clk_conf1.write_volatile(syscon_clk_conf1.read_volatile() | 1 << 18); // hw->clk_conf1.clk_bt_en = en;
        syscon_clk_conf.write_volatile(syscon_clk_conf.read_volatile() | 1 << 22); // hw->clk_conf.clk_etm_en = en;
        lpcon_clk_conf.write_volatile(lpcon_clk_conf.read_volatile() | 1 << 1); // hw->clk_conf.clk_coex_en = en
    }
}

/// Initialize the IEEE802.15.4 MAC
fn ieee802154_mac_init() {
    //TODO: need to be removed
    etm_clk_enable();

    ieee802154_pib_init();

    enable_events(Ieee802154Event::Ieee802154EventMask as u16); // ieee802154_hal_enable_events(IEEE802154_EVENT_MASK);

    if !get_test_mode() {
        disable_events(
            (Ieee802154Event::Ieee802154EventTimer0Overflow as u16)
                | (Ieee802154Event::Ieee802154EventTimer1Overflow as u16),
        );
    }

    enable_tx_abort_events(
        Ieee802154TxAbortReason::Ieee802154TxAbortByRxAckTimeout.bit()
            | Ieee802154TxAbortReason::Ieee802154TxAbortByTxCoexBreak.bit()
            | Ieee802154TxAbortReason::Ieee802154TxAbortByTxSecurityError.bit()
            | Ieee802154TxAbortReason::Ieee802154TxAbortByCcaFailed.bit()
            | Ieee802154TxAbortReason::Ieee802154TxAbortByCcaBusy.bit(),
    );
    enable_rx_abort_events(
        Ieee802154RxAbortReason::Ieee802154RxAbortByTxAckTimeout.bit()
            | Ieee802154RxAbortReason::Ieee802154RxAbortByTxAckCoexBreak.bit(),
    );

    set_ed_sample_mode(Ieee802154EdSampleMode::Ieee802154EdSampleAvg);

    set_ack_pti();
    ieee802154_set_txrx_pti(Ieee802154TxrxScene::Ieee802154SceneIdle);

    unsafe {
        bt_bb_set_zb_tx_on_delay(50); // set tx on delay for libbtbb.a
    }
    ieee802154()
        .rxon_delay
        .modify(|_, w| w.rxon_delay().variant(50));

    // esp_intr_alloc(ETS_ZB_MAC_SOURCE, 0, ieee802154_isr, NULL, NULL);
    esp32c6_hal::interrupt::enable(
        esp32c6_hal::peripherals::Interrupt::ZB_MAC,
        esp32c6_hal::interrupt::Priority::Priority1,
    )
    .unwrap();
    unsafe {
        esp32c6_hal::riscv::interrupt::enable();
    }

    // esp_receive_ack_timeout_timer_init(); // TODO timer stuff

    /*
        memset(rx_frame, 0, sizeof(rx_frame));
        ieee802154_state = IEEE802154_STATE_IDLE;
    */
}

/*
static bool start_ed(uint32_t duration)
{
    ieee802154_hal_enable_events(IEEE802154_EVENT_ED_DONE);
    ieee802154_hal_set_ed_duration(duration);
    ieee802154_hal_set_cmd(IEEE802154_CMD_ED_START);

    return true;
}
*/

pub fn tx_init(frame: *const u8) {
    let tx_frame = frame;
    // stop_current_operation();
    ieee802154_pib_update();
    ieee802154_sec_update();

    ieee802154_hal_set_tx_addr(tx_frame);

    if true
    /* ieee802154_frame_is_ack_required(frame) */
    {
        // set rx pointer for ack frame
        set_next_rx_buffer();
    }
}

pub fn ieee802154_transmit(frame: *const u8, cca: bool) -> i32 {
    critical_section::with(|cs| {
        tx_init(frame);

        ieee802154_set_txrx_pti(Ieee802154TxrxScene::Ieee802154SceneTx);

        if cca {
            // ieee802154_hal_disable_events(IEEE802154_EVENT_ED_DONE);
            // ieee802154_hal_set_cmd(IEEE802154_CMD_CCA_TX_START);
            // ieee802154_state = IEEE802154_STATE_TX_CCA;
        } else {
            ieee802154_hal_set_cmd(Ieee802154Cmd::Ieee802154CmdTxStart);
            // if (ieee802154_frame_get_type(frame) == IEEE802154_FRAME_TYPE_ACK
            //     && ieee802154_frame_get_version(frame) == IEEE802154_FRAME_VERSION_2)
            // {
            //     ieee802154_state = IEEE802154_STATE_TX_ENH_ACK;
            // } else {
            *STATE.borrow_ref_mut(cs) = Ieee802154State::Transmit;
            // }
        }
    });

    return 0; // ESP_OK;
}

pub fn ieee802154_receive() -> i32 {
    // if (((ieee802154_state == IEEE802154_STATE_RX)
    //     || (ieee802154_state == IEEE802154_STATE_TX_ACK))
    //     && (!ieee802154_pib_is_pending()))
    // {
    //     // already in rx state, don't abort current rx operation
    //     return ESP_OK;
    // }

    critical_section::with(|cs| {
        if *STATE.borrow_ref(cs) == Ieee802154State::Receive {
            return;
        }

        rx_init();

        enable_rx();

        *STATE.borrow_ref_mut(cs) = Ieee802154State::Receive;
    });

    return 0; // ESP-OK
}

fn rx_init() {
    // stop_current_operation();
    ieee802154_pib_update();
}

fn enable_rx() {
    set_next_rx_buffer();
    ieee802154_set_txrx_pti(Ieee802154TxrxScene::Ieee802154SceneRx);

    ieee802154_hal_set_cmd(Ieee802154Cmd::Ieee802154CmdRxStart);

    // ieee802154_state = IEEE802154_STATE_RX;
}

fn set_next_rx_buffer() {
    unsafe {
        ieee802154_hal_set_rx_addr(RX_BUFFER.as_mut_ptr() as *mut u8);
    }
}

pub fn set_promiscuous(enable: bool) {
    ieee802154_pib_set_promiscuous(enable);
}

pub fn set_auto_ack_tx(enable: bool) {
    ieee802154_pib_set_auto_ack_tx(enable);
}

pub fn set_auto_ack_rx(enable: bool) {
    ieee802154_pib_set_auto_ack_rx(enable);
}

pub fn set_enhance_ack_tx(enable: bool) {
    ieee802154_pib_set_enhance_ack_tx(enable);
}

pub fn set_coordinator(enable: bool) {
    ieee802154_pib_set_coordinator(enable);
}

pub fn set_rx_when_idle(enable: bool) {
    ieee802154_pib_set_rx_when_idle(enable);
}

pub fn set_tx_power(power: i8) {
    ieee802154_pib_set_tx_power(power);
}

pub fn set_channel(channel: u8) {
    ieee802154_pib_set_channel(channel);
}

pub fn set_pending_mode(mode: Ieee802154PendingMode) {
    ieee802154_pib_set_pending_mode(mode);
}

pub fn set_multipan_enable(mask: u8) {
    ieee802154_pib_set_multipan_enable(mask);
}

pub fn set_short_address(index: u8, address: u16) {
    ieee802154_pib_set_short_address(index, address);
}

pub fn set_extended_address(index: u8, address: [u8; IEEE802154_FRAME_EXT_ADDR_SIZE]) {
    ieee802154_pib_set_extended_address(index, address);
}

pub fn set_cca_theshold(cca_threshold: i8) {
    ieee802154_pib_set_cca_theshold(cca_threshold);
}

pub fn set_cca_mode(mode: Ieee802154CcaMode) {
    ieee802154_pib_set_cca_mode(mode);
}

pub fn set_panid(index: u8, id: u16) {
    ieee802154_pib_set_panid(index, id);
}

#[inline(always)]
fn ieee802154_sec_update() {
    let is_security = false;
    ieee802154_hal_set_transmit_security(is_security);
    // ieee802154_sec_clr_transmit_security();
}

// pub fn ieee802154_set_promiscuous(enable: bool) {
//     ieee802154_pib_set_promiscuous(enable);
//     ieee802154_pib_set_auto_ack_rx(!enable);
//     ieee802154_pib_set_auto_ack_tx(!enable);
// }

/// Enable the ETM clock
fn etm_clk_enable() {
    #[cfg(not(feature = "esp32c6"))]
    compile_error!("Unsupported target");

    const REG_MODEM_SYSCON_BASE: u32 = 0x600A9800;
    const MODEM_SYSCON_CLK_CONF_REG: u32 = REG_MODEM_SYSCON_BASE + 0x4;
    const MODEM_SYSCON_CLK_ETM_EN: u32 = 1 << 22;
    const ETM_REG_BASE: u32 = 0x600A8800;
    const ETM_CLK_EN_REG: u32 = ETM_REG_BASE + 0x008C;
    const ETM_CLK_EN: u32 = 1 << 0;
    const DR_REG_PCR_BASE: u32 = 0x60096000;
    const CLKRST_MODCLK_CONF_REG: u32 = DR_REG_PCR_BASE + 0x98;
    const PCR_ETM_CLK_EN_M: u32 = 1 << 0;
    const PCR_ETM_RST_EN: u32 = 1 << 1;

    unsafe {
        (MODEM_SYSCON_CLK_CONF_REG as *mut u32).write_volatile(
            (MODEM_SYSCON_CLK_CONF_REG as *mut u32).read_volatile() | MODEM_SYSCON_CLK_ETM_EN,
        );

        (ETM_CLK_EN_REG as *mut u32)
            .write_volatile((ETM_CLK_EN_REG as *mut u32).read_volatile() | ETM_CLK_EN);

        (CLKRST_MODCLK_CONF_REG as *mut u32).write_volatile(
            (CLKRST_MODCLK_CONF_REG as *mut u32).read_volatile()
                | PCR_ETM_CLK_EN_M
                | PCR_ETM_RST_EN,
        ); // Active ETM clock
    }
}

fn next_operation() {
    critical_section::with(|cs| {
        if ieee802154_pib_get_rx_when_idle() {
            enable_rx();
            *STATE.borrow_ref_mut(cs) = Ieee802154State::Receive;
        } else {
            *STATE.borrow_ref_mut(cs) = Ieee802154State::Idle;
        }
    });
}

use esp32c6_hal::prelude::interrupt;
#[interrupt]
fn ZB_MAC() {
    log::info!("ZB_MAC interrupt");

    let events = ieee802154_hal_get_events();
    ieee802154_hal_clear_events(events);

    log::info!("events = {:032b}", events);

    if events & (Ieee802154Event::Ieee802154EventRxSfdDone as u16) != 0 {
        // IEEE802154_STATE_TX && IEEE802154_STATE_TX_CCA && IEEE802154_STATE_TX_ENH_ACK for isr processing delay
        log::info!("rx sfd done");
    }

    if events & (Ieee802154Event::Ieee802154EventTxSfdDone as u16) != 0 {
        // IEEE802154_STATE_RX for isr processing delay, only 821
        // IEEE802154_STATE_TX_ACK for workaround jira ZB-81.
        log::info!("tx sfd done");
    }

    if events & (Ieee802154Event::Ieee802154EventTxDone as u16) != 0 {
        log::info!("tx done");
        next_operation();
    }

    if events & (Ieee802154Event::Ieee802154EventRxDone as u16) != 0 {
        log::info!("rx done");
        unsafe {
            log::info!("{:x?}", RX_BUFFER);
        }
        next_operation();
    }
}
