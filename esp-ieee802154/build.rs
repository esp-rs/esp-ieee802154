use std::{
    env,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

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

    #[cfg(not(any(feature = "esp32c6", feature = "esp32h2")))]
    {
        compile_error!("Either esp32c6 or esp32h2 needs to be selected via a feature");
    }

    #[cfg(feature = "esp32c6")]
    {
        copy_file!(out, "ld/esp32c6/rom_coexist.x");
        copy_file!(out, "ld/esp32c6/rom_functions.x");
        copy_file!(out, "ld/esp32c6/rom_phy.x");

        copy_file!(out, "libs/esp32c6/libbtbb.a");
        copy_file!(out, "libs/esp32c6/libphy.a");
        copy_file!(out, "libs/esp32c6/libcoexist.a");
    }

    #[cfg(feature = "esp32h2")]
    {
        empty_file(out, "rom_coexist.x");
        copy_file!(out, "ld/esp32h2/rom_functions.x");
        empty_file(out, "rom_phy.x");

        copy_file!(out, "libs/esp32h2/libbtbb.a");
        copy_file!(out, "libs/esp32h2/libphy.a");
        copy_file!(out, "libs/esp32h2/libcoexist.a");
    }

    println!("cargo:rustc-link-lib=btbb");
    println!("cargo:rustc-link-lib=coexist");
    println!("cargo:rustc-link-lib=phy");

    println!("cargo:rustc-link-search={}", out.display());
}

#[allow(unused)]
fn empty_file(path: &Path, filename: &str) {
    File::create(path.join(filename)).unwrap();
}
