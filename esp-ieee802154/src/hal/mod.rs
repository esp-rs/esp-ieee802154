use crate::pib::Ieee802154CcaMode;

#[cfg_attr(feature = "esp32c6", path = "ral/esp32c6.rs")]
#[cfg_attr(feature = "esp32h2", path = "ral/esp32h2.rs")]
mod ral;

/// IEEE802154 events
#[allow(unused)]
#[derive(Debug, Clone, Copy)]
pub enum Ieee802154Event {
    Ieee802154EventTxDone = 1 << 0,
    Ieee802154EventRxDone = 1 << 1,
    Ieee802154EventAckTxDone = 1 << 2,
    Ieee802154EventAckRxDone = 1 << 3,
    Ieee802154EventRxAbort = 1 << 4,
    Ieee802154EventTxAbort = 1 << 5,
    Ieee802154EventEdDone = 1 << 6,
    Ieee802154EventTimer0Overflow = 1 << 8,
    Ieee802154EventTimer1Overflow = 1 << 9,
    Ieee802154EventClockCountMatch = 1 << 10,
    Ieee802154EventTxSfdDone = 1 << 11,
    Ieee802154EventRxSfdDone = 1 << 12,
    Ieee802154EventMask = 0x00001FFF,
}

#[allow(unused)]
#[derive(Debug, Clone, Copy)]
pub enum Ieee802154TxAbortReason {
    Ieee802154TxAbortByRxAckStop = 1,
    Ieee802154TxAbortByRxAckSfdTimeout = 2,
    Ieee802154TxAbortByRxAckCrcError = 3,
    Ieee802154TxAbortByRxAckInvalidLen = 4,
    Ieee802154TxAbortByRxAckFilterFail = 5,
    Ieee802154TxAbortByRxAckNoRss = 6,
    Ieee802154TxAbortByRxAckCoexBreak = 7,
    Ieee802154TxAbortByRxAckTypeNotAck = 8,
    Ieee802154TxAbortByRxAckRestart = 9,
    Ieee802154TxAbortByRxAckTimeout = 16,
    Ieee802154TxAbortByTxStop = 17,
    Ieee802154TxAbortByTxCoexBreak = 18,
    Ieee802154TxAbortByTxSecurityError = 19,
    Ieee802154TxAbortByCcaFailed = 24,
    Ieee802154TxAbortByCcaBusy = 25,
}

impl Ieee802154TxAbortReason {
    pub fn bit(&self) -> u32 {
        1 << *self as u32 - 1
    }
}

#[allow(unused)]
#[derive(Debug, Clone, Copy)]
pub enum Ieee802154RxAbortReason {
    Ieee802154RxAbortByRxStop = 1,
    Ieee802154RxAbortBySfdTimeout = 2,
    Ieee802154RxAbortByCrcError = 3,
    Ieee802154RxAbortByInvalidLen = 4,
    Ieee802154RxAbortByFilterFail = 5,
    Ieee802154RxAbortByNoRss = 6,
    Ieee802154RxAbortByCoexBreak = 7,
    Ieee802154RxAbortByUnexpectedAck = 8,
    Ieee802154RxAbortByRxRestart = 9,
    Ieee802154RxAbortByTxAckTimeout = 16,
    Ieee802154RxAbortByTxAckStop = 17,
    Ieee802154RxAbortByTxAckCoexBreak = 18,
    Ieee802154RxAbortByEnhackSecurityError = 19,
    Ieee802154RxAbortByEdAbort = 24,
    Ieee802154RxAbortByEdStop = 25,
    Ieee802154RxAbortByEdCoexReject = 26,
}

impl Ieee802154RxAbortReason {
    pub fn bit(&self) -> u32 {
        1 << *self as u32 - 1
    }
}

#[allow(unused)]
#[derive(Debug, Clone, Copy)]
pub enum Ieee802154EdSampleMode {
    Ieee802154EdSampleMax = 0x00,
    Ieee802154EdSampleAvg = 0x01,
}

/// IEEE802154 commands
#[allow(unused)]
#[derive(Debug, Clone, Copy)]
pub enum Ieee802154Cmd {
    Ieee802154CmdTxStart = 0x41,
    Ieee802154CmdRxStart = 0x42,
    Ieee802154CmdCcaTxStart = 0x43,
    Ieee802154CmdEdStart = 0x44,
    Ieee802154CmdStop = 0x45,
    Ieee802154CmdDtmTxStart = 0x46,
    Ieee802154CmdDtmRxStart = 0x47,
    Ieee802154CmdDtmStop = 0x48,
    Ieee802154CmdTimer0Start = 0x4C,
    Ieee802154CmdTimer0Stop = 0x4D,
    Ieee802154CmdTimer1Start = 0x4E,
    Ieee802154CmdTimer1Stop = 0x4F,
}

#[allow(unused)]
#[derive(Debug, Clone, Copy)]
pub enum Ieee802154MultipanIndex {
    Ieee802154Multipan0 = 0,
    Ieee802154Multipan1 = 1,
    Ieee802154Multipan2 = 2,
    Ieee802154Multipan3 = 3,
    Ieee802154MultipanMax = 4,
}

impl From<usize> for Ieee802154MultipanIndex {
    fn from(value: usize) -> Self {
        match value {
            0 => Ieee802154MultipanIndex::Ieee802154Multipan0,
            1 => Ieee802154MultipanIndex::Ieee802154Multipan1,
            2 => Ieee802154MultipanIndex::Ieee802154Multipan2,
            3 => Ieee802154MultipanIndex::Ieee802154Multipan3,
            _ => panic!(),
        }
    }
}

pub fn ieee802154() -> &'static self::ral::ieee802154::RegisterBlock {
    unsafe { &*self::ral::IEEE802154::PTR }
}

#[inline(always)]
pub fn enable_events(events: u16) {
    ieee802154()
        .event_en
        .modify(|r, w| w.event_en().variant(r.event_en().bits() | events));
}

#[inline(always)]
pub fn disable_events(events: u16) {
    ieee802154()
        .event_en
        .modify(|r, w| w.event_en().variant(r.event_en().bits() & !events));
}

#[inline(always)]
pub fn enable_tx_abort_events(events: u32) {
    ieee802154().tx_abort_interrupt_control.modify(|r, w| {
        w.tx_abort_interrupt_control()
            .variant(r.tx_abort_interrupt_control().bits() | events)
    });
}

#[inline(always)]
pub fn enable_rx_abort_events(events: u32) {
    ieee802154().rx_abort_intr_ctrl.modify(|r, w| {
        w.rx_abort_intr_ctrl()
            .variant(r.rx_abort_intr_ctrl().bits() | events)
    });
}

#[inline(always)]
pub fn set_ed_sample_mode(ed_sample_mode: Ieee802154EdSampleMode) {
    ieee802154()
        .ed_scan_cfg
        .modify(|_, w| w.ed_sample_mode().variant(ed_sample_mode as u8));
}

#[allow(unused)]
#[inline(always)]
pub fn ieee802154_hal_set_normal_pti(normal_pti: u8) {
    ieee802154()
        .coex_pti
        .modify(|_, w| w.coex_pti().variant(normal_pti));
}

#[allow(unused)]
#[inline(always)]
pub fn ieee802154_hal_set_ack_pti(ack_pti: u8) {
    ieee802154()
        .coex_pti
        .modify(|_, w| w.coex_ack_pti().variant(ack_pti));
}

#[inline(always)]
pub fn ieee802154_hal_set_tx_addr(addr: *const u8) {
    ieee802154()
        .txdma_addr
        .modify(|_, w| w.txdma_addr().variant(addr as u32));
}

#[inline(always)]
pub fn ieee802154_hal_set_cmd(cmd: Ieee802154Cmd) {
    ieee802154()
        .command
        .modify(|_, w| w.opcode().variant(cmd as u8))
}

#[inline(always)]
pub fn ieee802154_hal_set_freq(freq: u8) {
    ieee802154().channel.modify(|_, w| w.hop().variant(freq));
}

#[inline(always)]
pub fn ieee802154_hal_get_freq() -> u8 {
    ieee802154().channel.read().hop().bits()
}

#[inline(always)]
pub fn ieee802154_hal_set_power(power: u8) {
    ieee802154()
        .tx_power
        .modify(|_, w| w.tx_power().variant(power));
}

#[inline(always)]
pub fn ieee802154_hal_set_multipan_enable_mask(mask: u8) {
    // apparently the REGS are garbage and the struct is right?
    ieee802154()
        .ctrl_cfg
        .modify(|r, w| unsafe { w.bits(r.bits() & !(0b1111 << 29) | (mask as u32) << 29) })
}

#[inline(always)]
pub fn ieee802154_hal_set_multipan_panid(index: Ieee802154MultipanIndex, panid: u16) {
    unsafe {
        let pan_id = ieee802154().inf0_pan_id.as_ptr().offset(4 * index as isize);
        pan_id.write_volatile(panid as u32);
    }
}

#[inline(always)]
pub fn ieee802154_hal_set_multipan_short_addr(index: Ieee802154MultipanIndex, value: u16) {
    unsafe {
        let short_addr = ieee802154()
            .inf0_short_addr
            .as_ptr()
            .offset(4 * index as isize);
        short_addr.write_volatile(value as u32);
    }
}

#[inline(always)]
pub fn ieee802154_hal_set_multipan_ext_addr(index: Ieee802154MultipanIndex, ext_addr: *const u8) {
    unsafe {
        let mut ext_addr_ptr = ieee802154()
            .inf0_extend_addr0
            .as_ptr()
            .offset(4 * index as isize);
        ext_addr_ptr.write_volatile(
            ((ext_addr.offset(0).read_volatile() as u32) << 0)
                | ((ext_addr.offset(1).read_volatile() as u32) << 8)
                | ((ext_addr.offset(2).read_volatile() as u32) << 16)
                | ((ext_addr.offset(3).read_volatile() as u32) << 24),
        );

        ext_addr_ptr = ext_addr_ptr.offset(1);
        ext_addr_ptr.write_volatile(
            ((ext_addr.offset(4).read_volatile() as u32) << 0)
                | ((ext_addr.offset(5).read_volatile() as u32) << 8)
                | ((ext_addr.offset(6).read_volatile() as u32) << 16)
                | ((ext_addr.offset(7).read_volatile() as u32) << 24),
        );
    }
}

#[inline(always)]
pub fn ieee802154_hal_set_cca_mode(cca_mode: Ieee802154CcaMode) {
    ieee802154()
        .ed_scan_cfg
        .modify(|_, w| w.cca_mode().variant(cca_mode as u8));
}

#[inline(always)]
pub fn ieee802154_hal_set_cca_threshold(cca_threshold: i8) {
    ieee802154()
        .ed_scan_cfg
        .modify(|_, w| w.cca_ed_threshold().variant(cca_threshold as u8))
}

#[inline(always)]
pub fn ieee802154_hal_set_tx_auto_ack(enable: bool) {
    ieee802154()
        .ctrl_cfg
        .modify(|_, w| w.hw_auto_ack_tx_en().variant(enable));
}

#[inline(always)]
pub fn ieee802154_hal_get_tx_auto_ack() -> bool {
    ieee802154()
        .ctrl_cfg
        .read()
        .hw_auto_ack_tx_en()
        .bit_is_set()
}

#[inline(always)]
pub fn ieee802154_hal_set_rx_auto_ack(enable: bool) {
    ieee802154()
        .ctrl_cfg
        .modify(|_, w| w.hw_auto_ack_rx_en().variant(enable));
}

#[inline(always)]
pub fn ieee802154_hal_set_tx_enhance_ack(enable: bool) {
    ieee802154()
        .ctrl_cfg
        .modify(|_, w| w.hw_enhance_ack_tx_en().variant(enable));
}

#[inline(always)]
pub fn ieee802154_hal_get_tx_enhance_ack() -> bool {
    ieee802154()
        .ctrl_cfg
        .read()
        .hw_enhance_ack_tx_en()
        .bit_is_set()
}

#[inline(always)]
pub fn ieee802154_hal_set_coordinator(enable: bool) {
    ieee802154()
        .ctrl_cfg
        .modify(|_, w| w.pan_coordinator().variant(enable));
}

#[inline(always)]
pub fn ieee802154_hal_set_promiscuous(enable: bool) {
    ieee802154()
        .ctrl_cfg
        .modify(|_, w| w.promiscuous_mode().variant(enable));
}

#[inline(always)]
pub fn ieee802154_hal_set_pending_mode(enable: bool) {
    ieee802154()
        .ctrl_cfg
        .modify(|_, w| w.autopend_enhance().variant(enable));
}

#[inline(always)]
pub fn ieee802154_hal_get_events() -> u16 {
    ieee802154().event_status.read().bits() as u16
}

#[inline(always)]
pub fn ieee802154_hal_clear_events(events: u16) {
    ieee802154()
        .event_status
        .modify(|r, w| unsafe { w.event_status().bits(r.event_status().bits() & events) });
}

#[inline(always)]
pub fn ieee802154_hal_set_transmit_security(enable: bool) {
    ieee802154().sec_ctrl.modify(|_, w| w.sec_en().bit(enable));
}

#[inline(always)]
pub fn ieee802154_hal_set_rx_addr(addr: *mut u8) {
    ieee802154()
        .rxdma_addr
        .modify(|_, w| w.rxdma_addr().variant(addr as u32));
}

#[allow(unused)]
#[inline(always)]
pub fn ieee802154_hal_set_pending_bit(enable: bool) {
    ieee802154()
        .ack_frame_pending_en
        .modify(|_, w| w.ack_frame_pending_en().variant(enable));
}
