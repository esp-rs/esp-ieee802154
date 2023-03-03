use core::cell::RefCell;

use critical_section::Mutex;

static TEST_MODE: Mutex<RefCell<bool>> = Mutex::new(RefCell::new(false));

pub fn set_test_mode(enable: bool) {
    critical_section::with(|cs| {
        *(TEST_MODE.borrow_ref_mut(cs)) = enable;
    });
}

pub fn get_test_mode() -> bool {
    critical_section::with(|cs| *(TEST_MODE.borrow_ref(cs)))
}

pub fn set_ack_pti() {
    extern "C" {
        fn esp_ieee802154_coex_pti_set(value: u8);
    }
    unsafe {
        // using this results in a linker error `g_coa_funcs_p` not found
        // esp_ieee802154_coex_pti_set(0); // what is IEEE802154_ACK ?????? define here: https://github.com/espressif/esp-idf/blob/master/components/esp_coex/include/esp_coex_i154.h
    }
}
