use crate::utils::ieee802154;

/* IEEE802154 events */
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

#[derive(Debug, Clone, Copy)]
pub enum Ieee802154EdSampleMode {
    Ieee802154EdSampleMax = 0x00,
    Ieee802154EdSampleAvg = 0x01,
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
