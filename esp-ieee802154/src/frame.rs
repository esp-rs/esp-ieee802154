pub const IEEE802154_FRAME_AR_OFFSET: usize = 1;
pub const IEEE802154_FRAME_AR_BIT: u8 = 1 << 5;
pub const IEEE802154_FRAME_VERSION_OFFSET: usize = 2;
pub const IEEE802154_FRAME_VERSION_MASK: u8 = 0x30;
pub const IEEE802154_FRAME_VERSION_1: u8 = 0x10; // IEEE 802.15.4 - 2006 & 2011
pub const IEEE802154_FRAME_VERSION_2: u8 = 0x20; // IEEE 802.15.4 - 2015

pub fn ieee802154_frame_is_ack_required(frame: &[u8]) -> bool {
    (frame[IEEE802154_FRAME_AR_OFFSET] & IEEE802154_FRAME_AR_BIT) != 0
}

pub fn ieee802154_frame_get_version(frame: &[u8]) -> u8 {
    frame[IEEE802154_FRAME_VERSION_OFFSET] & IEEE802154_FRAME_VERSION_MASK
}
