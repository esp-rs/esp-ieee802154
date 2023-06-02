use std::{env, fs::File, io::Write, path::PathBuf};

macro_rules! copy_file {
    // $out:  `&PathBuf`
    // $from: &'static str
    ( $out:expr, $from:expr ) => {{
        let (_path, fname) = $from.rsplit_once('/').unwrap();
        File::create($out.join(fname))
            .unwrap()
            .write_all(include_bytes!($from))
            .unwrap()
    }};
}

fn main() {
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());

    #[cfg(feature = "esp32c6")]
    {
        copy_file!(out, "ld/esp32c6/rom_functions.x");
        copy_file!(out, "libs/esp32c6/libbtbb.a");
        copy_file!(out, "libs/esp32c6/libphy.a");
        copy_file!(out, "libs/esp32c6/libcoexist.a");
    }

    #[cfg(feature = "esp32h2")]
    {
        copy_file!(out, "ld/esp32h2/rom_functions.x");
        copy_file!(out, "libs/esp32h2/libbtbb.a");
        copy_file!(out, "libs/esp32h2/libphy.a");
    }

    println!("cargo:rustc-link-lib={}", "btbb");
    println!("cargo:rustc-link-lib={}", "phy");
    println!("cargo:rustc-link-lib={}", "coexist");
    println!("cargo:rustc-link-search={}", out.display());
}
