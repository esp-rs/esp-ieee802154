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
    let mut system = peripherals.PCR.split();
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

    println!("Start");
    esp_ieee802154_enable(&mut system.radio_clock_control);
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
            54, 0x41, 0x88, seq_number, 0xff, 0xff, 0xff, 0xff, 0x00, 0x00, //
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

#[no_mangle]
extern "C" fn rtc_clk_xtal_freq_get() -> i32 {
    0
}
