#![no_std]
#![feature(c_variadic)]

use hal::disable_events;
use pib::ieee802154_pib_init;
use util::{get_test_mode, set_ack_pti};

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

/// Enable the IEEE802.15.4 radio
pub fn esp_ieee802154_enable() {
    esp_phy_enable();
    ieee802154_enable();
    ieee802154_mac_init();
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
    // ieee802154_set_txrx_pti(IEEE802154_SCENE_IDLE);
    // #elif CONFIG_IDF_TARGET_ESP32H2
    //     REG_WRITE(IEEE802154_COEX_PTI_REG, 0x86);
    // #endif

    /*
        #if CONFIG_IDF_ENV_FPGA
            bt_bb_set_zb_tx_on_delay(80); // set tx on delay for libbtbb.a
        #else
            bt_bb_set_zb_tx_on_delay(50); // set tx on delay for libbtbb.a
            REG_WRITE(IEEE802154_RXON_DELAY_REG, 50);
        #endif

        #if CONFIG_IDF_TARGET_ESP32H4
            esp_intr_alloc(ETS_IEEE802154MAC_INTR_SOURCE, 0, ieee802154_isr, NULL, NULL);
        #elif CONFIG_IDF_TARGET_ESP32C6
            esp_intr_alloc(ETS_ZB_MAC_SOURCE, 0, ieee802154_isr, NULL, NULL);
        #elif CONFIG_IDF_TARGET_ESP32H2
            esp_intr_alloc(ETS_ZB_MAC_INTR_SOURCE, 0, ieee802154_isr, NULL, NULL);
        #endif

        esp_receive_ack_timeout_timer_init();
        memset(rx_frame, 0, sizeof(rx_frame));
        ieee802154_state = IEEE802154_STATE_IDLE;
    */
}

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
