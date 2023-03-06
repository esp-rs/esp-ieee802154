#![no_std]
#![no_main]

use esp32c6_hal::{
    clock::{ClockControl, CpuClock},
    peripherals::Peripherals,
    prelude::*,
    timer::TimerGroup,
    Delay, Rtc, Uart,
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

    let mut uart0 = Uart::new(peripherals.UART0);
    let mut rtc = Rtc::new(peripherals.LP_CLKRST);
    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    let mut timer0 = timer_group0.timer0;
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
        const DR_REG_PLIC_MX_BASE: u32 = 0x20001000;
        const PLIC_MXINT0_PRI_REG: u32 = DR_REG_PLIC_MX_BASE + 0x10;
        println!(
            "interrupt WIFI BB {}",
            ((PLIC_MXINT0_PRI_REG + 3 * 4) as *mut u32).read_volatile()
        );
        ((PLIC_MXINT0_PRI_REG + 3 * 4) as *mut u32).write_volatile(0);

        // try to disable the WIFI_BB interrupt
        println!(
            "interrupt WIFI BB {}",
            ((0x60010000 + 3 * 4) as *mut u32).read_volatile()
        );
        ((0x60010000 + 3 * 4) as *mut u32).write_volatile(0);
    }
    // esp32c6_hal::interrupt::enable(
    //     esp32c6_hal::peripherals::Interrupt::WIFI_BB,
    //     esp32c6_hal::interrupt::Priority::Priority1,
    // );

    let mut delay = Delay::new(&clocks);
    // loop {
    //     println!("call transmit");
    //     let frame = &[
    //         // 0x41, 0x88, 0, 0xCA, 0xDE, b'W', b'A', b'V', b'E', 0xE0, 0, 0,
    //         14 * 3 + 12, // LEN?
    //         0x41,
    //         0x88,
    //         0x40,
    //         0xff,
    //         0x01,
    //         0xff,
    //         0xff,
    //         0x00,
    //         0x00,
    //         0x08,
    //         0x02,
    //         0xff,
    //         0xff,
    //         0x00,
    //         0x00,
    //         0x0a,
    //         0xdb,
    //         0x28,
    //         0x0c,
    //         0x00,
    //         0x00,
    //         0x00,
    //         0x58,
    //         0xc5,
    //         0x0d,
    //         0x00,
    //         0x00,
    //         0x6f,
    //         0x0d,
    //         0x00,
    //         0x00,
    //         0xf0,
    //         0x98,
    //         0xeb,
    //         0x0e,
    //         0x55,
    //         0x79,
    //         0x1a,
    //         0x23,
    //         0x0d,
    //         0xd3,
    //         0x87,
    //         0x05,
    //         0x1a,
    //         0xd8,
    //         0x17,
    //         0x4a,
    //         0xcc,
    //         0xdc,
    //         0x7c,
    //         0x09,
    //         0x70,
    //         0xc7,
    //         0x84,
    //     ];
    //     ieee802154_transmit(frame.as_ptr() as *const u8, false);
    //     println!("called transmit");

    //     delay.delay_ms(1000u32);
    // }

    println!("before receive");
    ieee802154_receive();
    println!("after receive");

    // TODO should call receive again once we got something - otherwise we won't get anything more
    loop {}
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

#[interrupt]
fn WIFI_BB() {
    //    println!("WIFI BB interrupt");
}
