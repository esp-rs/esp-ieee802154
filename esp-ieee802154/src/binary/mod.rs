pub(crate) mod c_types;
#[cfg_attr(feature = "esp32h4", path = "include/esp32h4.rs")]
#[cfg_attr(feature = "esp32c6", path = "include/esp32c6.rs")]
pub(crate) mod include;
