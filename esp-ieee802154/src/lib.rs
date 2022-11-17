#![no_std]
#![feature(c_variadic)]

use binary::include_esp32h4::esp_phy_calibration_mode_t_PHY_RF_CAL_FULL;
use binary::include_esp32h4::register_chipv7_phy;
use hal::enable_events;
use hal::Ieee802154Event;

use crate::utils::clkrst;
use crate::utils::etm;

mod binary;
mod compat;
mod hal;
mod ral;
mod utils;

extern "C" {
    pub fn bt_bb_v2_init_cmplx(print_version: u32); // from libbtbb.a

    pub fn bt_bb_set_zb_tx_on_delay(time: u16); // from libbtbb.a

    pub fn phy_version_print(); // from libphy.a
}

const PHY_ENABLE_VERSION_PRINT: u32 = 1;

pub fn esp_ieee802154_enable() {
    esp_phy_enable();
    ieee802154_enable();
    ieee802154_mac_init();
}

fn esp_phy_enable() {
    //_lock_acquire(&s_phy_access_lock);
    //if (s_phy_access_ref == 0) {
    unsafe {
        register_chipv7_phy(
            core::ptr::null(),
            core::ptr::null_mut(),
            esp_phy_calibration_mode_t_PHY_RF_CAL_FULL,
        );
        bt_bb_v2_init_cmplx(PHY_ENABLE_VERSION_PRINT); // bt_bb_v2_init_cmplx // bt_bb_v2_init_cmplx(int print_version);
        phy_version_print(); // libphy.a // extern void phy_version_print(void);
    }
    //}

    //s_phy_access_ref++;

    //_lock_release(&s_phy_access_lock);
    // ESP32H4-TODO: enable common clk.
}

fn ieee802154_enable() {
    // TODO hopefully these will be in PAC

    // REG_SET_BIT(SYSTEM_MODEM_CLK_EN_REG, SYSTEM_IEEE802154MAC_CLK_EN);
    // REG_SET_BIT(SYSTEM_MODEM_CLK_EN_REG, SYSTEM_IEEE802154BB_CLK_EN);
    // REG_SET_BIT(SYSTEM_MODEM_RST_EN_REG, SYSTEM_IEEE802154MAC_RST);
    // REG_SET_BIT(SYSTEM_MODEM_RST_EN_REG, SYSTEM_IEEE802154BB_RST);
    // REG_CLR_BIT(SYSTEM_MODEM_RST_EN_REG, SYSTEM_IEEE802154MAC_RST);
    // REG_CLR_BIT(SYSTEM_MODEM_RST_EN_REG, SYSTEM_IEEE802154BB_RST);
}

fn ieee802154_mac_init() {
    //TODO: need to be removed
    etm_clk_enable();

    /*
        ieee802154_pib_init();
    */

    enable_events(Ieee802154Event::Ieee802154EventMask as u16); // ieee802154_hal_enable_events(IEEE802154_EVENT_MASK);

    /*
        if(!get_test_mode()) {
            ieee802154_hal_disable_events((IEEE802154_EVENT_TIMER0_OVERFLOW) | (IEEE802154_EVENT_TIMER1_OVERFLOW));
        }

        ieee802154_hal_enable_tx_abort_events(BIT(IEEE802154_TX_ABORT_BY_RX_ACK_TIMEOUT - 1) | BIT(IEEE802154_TX_ABORT_BY_TX_COEX_BREAK - 1) | BIT(IEEE802154_TX_ABORT_BY_TX_SECURITY_ERROR - 1) | BIT(IEEE802154_TX_ABORT_BY_CCA_FAILED - 1) | BIT(IEEE802154_TX_ABORT_BY_CCA_BUSY - 1));
        ieee802154_hal_enable_rx_abort_events(BIT(IEEE802154_RX_ABORT_BY_TX_ACK_TIMEOUT - 1) | BIT(IEEE802154_RX_ABORT_BY_TX_ACK_COEX_BREAK - 1));

        ieee802154_hal_set_ed_sample_mode(IEEE802154_ED_SAMPLE_AVG);

        REG_WRITE(IEEE802154_COEX_PTI_REG, 0x6);

        bt_bb_set_zb_tx_on_delay(80); // set tx on delay for libbtbb.a

        #if CONFIG_IDF_TARGET_ESP32H4
            // TODO: need support for ESP32C6
            esp_intr_alloc(ETS_IEEE802154MAC_INTR_SOURCE, 0, ieee802154_isr, NULL, NULL);
        #elif CONFIG_IDF_TARGET_ESP32C6
            // TODO: need support for ESP32C6
        #endif
        memset(rx_frame, 0, sizeof(rx_frame));
        ieee802154_state = IEEE802154_STATE_IDLE;
    */
}

fn etm_clk_enable() {
    //#if CONFIG_IDF_TARGET_ESP32H4_BETA_VERSION_2
    //    REG_SET_BIT(SYSTEM_PERIP_CLK_EN1_REG, SYSTEM_ETM_CLK_EN);
    etm().etm_clk_en.modify(|_, w| unsafe { w.bits(1) });
    clkrst().clkrst_modclk_conf.modify(|_, w| {
        w.clkrst_etm_clk_sel()
            .set_bit()
            .clkrst_etm_clk_active()
            .set_bit()
    });
    //#endif
}
