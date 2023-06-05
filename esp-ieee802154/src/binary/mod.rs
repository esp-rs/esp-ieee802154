pub(crate) mod c_types;
#[cfg_attr(feature = "esp32c6", path = "include/esp32c6.rs")]
#[cfg_attr(feature = "esp32h2", path = "include/esp32h2.rs")]
pub(crate) mod include;
