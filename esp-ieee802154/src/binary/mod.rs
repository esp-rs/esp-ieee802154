#[cfg_attr(feature = "esp32c6", path = "include/esp32c6.rs")]
#[cfg_attr(feature = "esp32h2", path = "include/esp32h2.rs")]
pub(crate) mod include;

#[allow(non_camel_case_types)]
mod c_types {
    pub type c_uint = u32;
    pub type c_int = i32;
    pub type c_ulonglong = u32;
    pub type c_longlong = i32;
    pub type c_uchar = u8;

    pub type c_short = i16;
    pub type c_ushort = u16;
    pub type c_schar = i8;
    pub type c_char = u8;
    pub type c_long = i32;
    pub type c_ulong = u32;

    pub enum c_void {}
}
