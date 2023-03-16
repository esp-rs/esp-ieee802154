#![no_std]
#![no_main]

use esp32c6_hal::{
    clock::{ClockControl, CpuClock},
    peripherals::Peripherals,
    prelude::*,
    timer::TimerGroup,
    Delay, Rtc,
};
use esp_backtrace as _;
use esp_ieee802154::*;
use esp_println::println;

#[entry]
fn main() -> ! {
    esp_println::logger::init_logger(log::LevelFilter::Trace);

    let peripherals = Peripherals::take();
    let system = peripherals.PCR.split();
    let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock160MHz).freeze();

    let mut rtc = Rtc::new(peripherals.LP_CLKRST);
    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    let mut wdt0 = timer_group0.wdt;
    let timer_group1 = TimerGroup::new(peripherals.TIMG1, &clocks);
    let mut wdt1 = timer_group1.wdt;

    // Disable watchdog timers
    rtc.swd.disable();
    rtc.rwdt.disable();
    wdt0.disable();
    wdt1.disable();

    setup();
    wifi_clock_enable();
    phy_enable_clock();

    println!("Start");

    esp_ieee802154_enable();
    println!("Enabled");

    unsafe {
        // map WIFI_BB to the masked interrupt 31 to disable it
        // not doing that will freeze up the code (since it's endless calling the DefaultHandler)
        // uncommenting WIFI_BB at the bottom makes it work (but the ISR is called)
        (0x60010000 as *mut u32).offset(3).write_volatile(31);
    }

    let mut delay = Delay::new(&clocks);
    delay.delay_ms(10u32);

    set_channel(11);
    set_promiscuous(false);
    set_panid(0, 0x4242);
    set_short_address(0, 0x2323);

    let mut seq_number = 0u8;
    loop {
        // data need to be in SRAM
        let mut frame: [u8; 55] = [
            54, 0x41, 0x88, seq_number, 0xff, 0x01, 0xff, 0xff, 0x00, 0x00, //
            b'H', b'e', b'l', b'l', b'o', b' ', b'W', b'o', b'r', b'l', b'd', b'!', b'H', b'e',
            b'l', b'l', b'o', b' ', b'E', b'S', b'P', b'3', b'2', b'-', b'C', b'6', b'!', b' ',
            b'8', b'0', b'2', b'.', b'1', b'5', b'.', b'4', b' ', b't', b'e', b's', b't', b'!',
            b'!', //
            0x00, 0x00,
        ];
        println!("FRAME: {:02x?}", frame);
        println!("frame addr = {:x}", frame.as_ptr() as u32);
        println!("call transmit");
        ieee802154_transmit(frame.as_ptr() as *const u8, false);
        println!("called transmit");
        println!();

        delay.delay_ms(1000u32);

        seq_number = seq_number.wrapping_add(1);
    }
}

pub(crate) fn setup() {
    unsafe {
        let pmu = &*esp32c6::PMU::PTR;

        pmu.hp_sleep_icg_modem
            .modify(|_, w| w.hp_sleep_dig_icg_modem_code().variant(0));
        pmu.hp_modem_icg_modem
            .modify(|_, w| w.hp_modem_dig_icg_modem_code().variant(1));
        pmu.hp_active_icg_modem
            .modify(|_, w| w.hp_active_dig_icg_modem_code().variant(2));
        pmu.imm_modem_icg
            .as_ptr()
            .write_volatile(pmu.imm_modem_icg.as_ptr().read_volatile() | 1 << 31);
        pmu.imm_sleep_sysclk
            .as_ptr()
            .write_volatile(pmu.imm_sleep_sysclk.as_ptr().read_volatile() | 1 << 28);

        let syscon_clk_conf_power_st = (0x600A9800 + 12) as *mut u32;
        syscon_clk_conf_power_st.write_volatile(syscon_clk_conf_power_st.read_volatile() | 6 << 28);
        syscon_clk_conf_power_st.write_volatile(syscon_clk_conf_power_st.read_volatile() | 4 << 24);
        syscon_clk_conf_power_st.write_volatile(syscon_clk_conf_power_st.read_volatile() | 6 << 20);
        syscon_clk_conf_power_st.write_volatile(syscon_clk_conf_power_st.read_volatile() | 6 << 16);
        syscon_clk_conf_power_st.write_volatile(syscon_clk_conf_power_st.read_volatile() | 6 << 12);
        syscon_clk_conf_power_st.write_volatile(syscon_clk_conf_power_st.read_volatile() | 6 << 8);

        let lp_clk_conf_power_st = (MODEM_LPCON + 8 * 4) as *mut u32;
        lp_clk_conf_power_st.write_volatile(lp_clk_conf_power_st.read_volatile() | 6 << 28);
        lp_clk_conf_power_st.write_volatile(lp_clk_conf_power_st.read_volatile() | 6 << 24);
        lp_clk_conf_power_st.write_volatile(lp_clk_conf_power_st.read_volatile() | 6 << 20);
        lp_clk_conf_power_st.write_volatile(lp_clk_conf_power_st.read_volatile() | 6 << 16);

        const MODEM_LPCON: u32 = 0x600AF000;
        let wifi_lp_clk_con = (MODEM_LPCON + 4 * 3) as *mut u32;
        const CLK_WIFIPWR_LP_SEL_OSC_SLOW: u32 = 0;
        const CLK_WIFIPWR_LP_SEL_OSC_FAST: u32 = 1;
        const CLK_WIFIPWR_LP_SEL_XTAL32K: u32 = 3;
        const CLK_WIFIPWR_LP_SEL_XTAL: u32 = 2;
        const CLK_WIFIPWR_LP_DIV_NUM_SHIFT: u32 = 4;
        const CLK_WIFIPWR_LP_DIV_NUM_MASK: u32 = 0b1111_1111_1111;
        const CLK_WIFIPWR_EN: u32 = 0;

        // modem_clock_hal_deselect_all_wifi_lpclk_source
        wifi_lp_clk_con.write_volatile(
            wifi_lp_clk_con.read_volatile()
                & !(1 << CLK_WIFIPWR_LP_SEL_OSC_SLOW
                    | 1 << CLK_WIFIPWR_LP_SEL_OSC_FAST
                    | 1 << CLK_WIFIPWR_LP_SEL_XTAL32K
                    | 1 << CLK_WIFIPWR_LP_SEL_XTAL),
        );

        // modem_clock_hal_select_wifi_lpclk_source
        wifi_lp_clk_con
            .write_volatile(wifi_lp_clk_con.read_volatile() | 1 << CLK_WIFIPWR_LP_SEL_OSC_SLOW);

        // modem_lpcon_ll_set_wifi_lpclk_divisor_value
        wifi_lp_clk_con.write_volatile(
            wifi_lp_clk_con.read_volatile()
                & !(CLK_WIFIPWR_LP_DIV_NUM_MASK << CLK_WIFIPWR_LP_DIV_NUM_SHIFT)
                | 0 << CLK_WIFIPWR_LP_DIV_NUM_SHIFT,
        );

        // modem_lpcon_ll_enable_wifipwr_clock
        let clk_conf = (MODEM_LPCON + 6 * 4) as *mut u32;
        clk_conf.write_volatile(clk_conf.read_volatile() | 1 << CLK_WIFIPWR_EN);
    }
}

pub(crate) fn wifi_clock_enable() {
    log::trace!("wifi_clock_enable");

    const MODEM_SYSCON: u32 = 0x600A9800;
    const CLK_CONF1: u32 = 5 * 4;
    const MODEM_LPCON: u32 = 0x600AF000;
    const CLK_CONF: u32 = 6 * 4;

    unsafe {
        let magic = (MODEM_SYSCON + 4) as *mut u32;
        magic.write_volatile(
            magic.read_unaligned() |
            1 << 23 | // clk_zb_apb_en
            1 << 24, // clk_zb_mac_en
        );

        let clk_conf1 = (MODEM_SYSCON + CLK_CONF1) as *mut u32;
        clk_conf1.write_volatile(
            clk_conf1.read_volatile() |
                1 << 10 // clk_wifi_apb_en
                | 1 << 9 // clk_wifimac_en
                | 1 << 16 // fe_apb_clock
                | 1 << 15 // fe_cal_160m_clock
                | 1 << 14 // fe_160m_clock
                | 1 << 13 // fe_80m_clock
                | 1 << 8 // wifibb_160x1_clock
                | 1 << 7 // wifibb_80x1_clock
                | 1 << 6 // wifibb_40x1_clock
                | 1 << 5 // wifibb_80x_clock
                | 1 << 4 // wifibb_40x_clock
                | 1 << 3 // wifibb_80m_clock
                | 1 << 2 // wifibb_44m_clock
                | 1 << 1 // wifibb_40m_clock
                | 1 << 0, // wifibb_22m_clock
        );

        let clk_conf = (MODEM_LPCON + CLK_CONF) as *mut u32;
        clk_conf.write_volatile(
            clk_conf.read_volatile() |
                1 << 0 // clk_wifipwr_en
                | 1 << 1, // enable_coex_clock
        );
    }
}

pub(crate) fn phy_enable_clock() {
    log::trace!("phy_enable_clock");
    const MODEM_LPCON: u32 = 0x600AF000;
    const CLK_CONF: u32 = MODEM_LPCON + 6 * 4;
    const I2C_MST_CLK_CONF: u32 = MODEM_LPCON + 4 * 4;

    unsafe {
        let clk_conf = CLK_CONF as *mut u32;
        clk_conf.write_volatile(
            clk_conf.read_volatile() | 1 << 2, // clk_i2c_mst_en
        );

        let i2c_mst_clk_conf = I2C_MST_CLK_CONF as *mut u32;
        i2c_mst_clk_conf.write_volatile(
            i2c_mst_clk_conf.read_volatile() | 1 << 0, // clk_i2c_mst_sel_160m
        );
    }

    log::trace!("phy_enable_clock done!");
}

#[no_mangle]
extern "C" fn rtc_clk_xtal_freq_get() -> i32 {
    0
}
