use crate::utils::StrBuf;

#[no_mangle]
pub unsafe extern "C" fn phy_printf(format: *const u8, _args: ...) {
    let fmt_str_ptr = format;
    let strbuf = StrBuf::from(fmt_str_ptr);
    log::info!("{}", strbuf.as_str_ref());
}

#[no_mangle]
pub unsafe extern "C" fn rtc_printf(format: *const u8, _args: ...) {
    let fmt_str_ptr = format;
    let strbuf = StrBuf::from(fmt_str_ptr);
    log::info!("{}", strbuf.as_str_ref());
}
