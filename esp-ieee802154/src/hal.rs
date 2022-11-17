use crate::utils::ieee802154;

/* IEEE802154 events */
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

#[inline(always)]
pub fn enable_events(events: u16) {
    ieee802154()
        .event_en
        .modify(|r, w| w.event_en().variant(r.event_en().bits() | events));
}
