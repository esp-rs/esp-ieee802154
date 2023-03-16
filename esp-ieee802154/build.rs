use std::{env, fs::File, io::Write, path::PathBuf};

fn main() {
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());

    #[cfg(feature = "esp32h2")]
    {
        copy(
            out,
            include_bytes!("ld/esp32h2/rom_functions.x"),
            "rom_functions.x",
        );
        copy(out, include_bytes!("libs/esp32h2/libbtbb.a"), "libbtbb.a");
        copy(out, include_bytes!("libs/esp32h2/libphy.a"), "libphy.a");
    }

    #[cfg(feature = "esp32c6")]
    {
        copy(
            out,
            include_bytes!("ld/esp32c6/rom_functions.x"),
            "rom_functions.x",
        );
        copy(out, include_bytes!("libs/esp32c6/libbtbb.a"), "libbtbb.a");
        copy(out, include_bytes!("libs/esp32c6/libphy.a"), "libphy.a");
        copy(
            out,
            include_bytes!("libs/esp32c6/libcoexist.a"),
            "libcoexist.a",
        );
    }

    println!("cargo:rustc-link-lib={}", "btbb");
    println!("cargo:rustc-link-lib={}", "phy");
    println!("cargo:rustc-link-lib={}", "coexist");
    println!("cargo:rustc-link-search={}", out.display());
}

#[allow(unused)]
fn copy(path: &PathBuf, data: &[u8], name: &str) {
    File::create(path.join(name))
        .unwrap()
        .write_all(data)
        .unwrap();
}
