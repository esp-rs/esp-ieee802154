#![no_std]
#![no_main]

#[cfg(feature = "esp32c6")]
use esp32c6_hal as esp_hal;
#[cfg(feature = "esp32h2")]
use esp32h2_hal as esp_hal;
use esp_backtrace as _;
use esp_hal::{
    clock::{ClockControl, CpuClock},
    peripherals::Peripherals,
    prelude::*,
    reset::software_reset,
    timer::TimerGroup,
    Rtc, Uart,
};
use esp_ieee802154::*;
use esp_println::println;

#[entry]
fn main() -> ! {
    esp_println::logger::init_logger(log::LevelFilter::Info);

    let peripherals = Peripherals::take();
    let mut system = peripherals.PCR.split();
    #[cfg(feature = "esp32c6")]
    let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock160MHz).freeze();
    #[cfg(feature = "esp32h2")]
    let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock96MHz).freeze();

    let mut rtc = Rtc::new(peripherals.LP_CLKRST);
    let timer_group0 = TimerGroup::new(
        peripherals.TIMG0,
        &clocks,
        &mut system.peripheral_clock_control,
    );
    let mut wdt0 = timer_group0.wdt;
    let timer_group1 = TimerGroup::new(
        peripherals.TIMG1,
        &clocks,
        &mut system.peripheral_clock_control,
    );
    let mut wdt1 = timer_group1.wdt;

    // Disable watchdog timers
    rtc.swd.disable();
    rtc.rwdt.disable();
    wdt0.disable();
    wdt1.disable();

    let mut uart0 = Uart::new(peripherals.UART0, &mut system.peripheral_clock_control);
    let mut cnt = 0;
    let mut read = [0u8; 2];
    loop {
        if let nb::Result::Ok(c) = uart0.read() {
            if c == b'r' {
                continue;
            }

            read[cnt] = c;
            cnt += 1;

            if cnt >= 2 {
                break;
            }
        }
    }
    let channel: u8 = unsafe { core::str::from_utf8_unchecked(&read) }
        .parse()
        .unwrap();

    let (.., radio) = peripherals.RADIO.split();
    let mut ieee802154 = Ieee802154::new(radio, &mut system.radio_clock_control);

    ieee802154.set_config(Config {
        channel,
        promiscuous: true,
        rx_when_idle: true,
        auto_ack_rx: false,
        auto_ack_tx: false,
        ..Config::default()
    });

    ieee802154.start_receive();

    loop {
        if let Some(frame) = ieee802154.get_raw_received() {
            println!("@RAW {:02x?}", &frame.data);
        }

        if let nb::Result::Ok(c) = uart0.read() {
            if c == b'r' {
                software_reset();
            }
        }
    }
}
