use std::{env, path::PathBuf};

fn main() {
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    println!("cargo:rustc-link-search={}", out.display());

    #[cfg(not(any(feature = "esp32c6", feature = "esp32h2")))]
    {
        compile_error!("Either esp32c6 or esp32h2 needs to be selected via a feature");
    }
}
