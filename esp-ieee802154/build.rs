use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() -> Result<(), String> {
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());

    copy(
        out,
        include_bytes!("ld/esp32h4/rom_functions.x"),
        "rom_functions.x",
    );
    copy(out, include_bytes!("libs/esp32h4/libbtbb.a"), "libbtbb.a");
    copy(out, include_bytes!("libs/esp32h4/libphy.a"), "libphy.a");

    println!("cargo:rustc-link-lib={}", "btbb");
    println!("cargo:rustc-link-lib={}", "phy");
    println!("cargo:rustc-link-search={}", out.display());

    Ok(())
}

fn copy(path: &PathBuf, data: &[u8], name: &str) {
    File::create(path.join(name))
        .unwrap()
        .write_all(data)
        .unwrap();
}
