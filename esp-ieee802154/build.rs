use std::{env, fs::File, io::Write, path::PathBuf};

fn main() {
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());

    #[cfg(feature = "esp32h4")]
    {
        copy(
            out,
            include_bytes!("ld/esp32h4/rom_functions.x"),
            "rom_functions.x",
        );
        copy(out, include_bytes!("libs/esp32h4/libbtbb.a"), "libbtbb.a");
        copy(out, include_bytes!("libs/esp32h4/libphy.a"), "libphy.a");
    }

    println!("cargo:rustc-link-lib={}", "btbb");
    println!("cargo:rustc-link-lib={}", "phy");
    println!("cargo:rustc-link-search={}", out.display());
}

#[allow(unused)]
fn copy(path: &PathBuf, data: &[u8], name: &str) {
    File::create(path.join(name))
        .unwrap()
        .write_all(data)
        .unwrap();
}
