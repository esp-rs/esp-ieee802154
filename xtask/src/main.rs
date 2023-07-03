use std::{
    env,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::Result;
use bindgen::Builder;
use clap::{Parser, ValueEnum};
use directories::BaseDirs;
use log::{info, LevelFilter};
use strum::Display;
use svd2rust::{generate::device::render, load_from, Config, Target};
use svdtools::patch::process_file;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, ValueEnum)]
#[strum(serialize_all = "lowercase")]
enum Chip {
    /// ESP32-C6
    Esp32c6,
    /// ESP32-H2
    Esp32h2,
}

#[derive(Debug, Parser)]
enum Cli {
    /// Generate the Register Access Layer (RAL)
    Ral {
        #[clap(value_enum)]
        chip: Chip,
    },

    /// Generate the binary includes using `bindgen`
    Includes {
        #[clap(value_enum)]
        chip: Chip,
    },
}

fn main() -> Result<()> {
    env_logger::Builder::new()
        .filter_module("xtask", LevelFilter::Info)
        .init();

    let args = Cli::parse();

    // The directory containing the cargo manifest for the 'xtask' package is a
    // subdirectory within the cargo workspace.
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace = workspace.parent().unwrap().canonicalize()?;

    match args {
        Cli::Ral { chip } => generate_register_access_layer(&workspace, chip),
        Cli::Includes { chip } => generate_binary_includes(&workspace, chip),
    }
}

fn generate_register_access_layer(workspace: &Path, chip: Chip) -> Result<()> {
    let svd_path = workspace.join("svd").join(chip.to_string());
    let svd_file = svd_path.join("ieee802154.svd");
    let out_dir = workspace
        .join("esp-ieee802154")
        .join("src")
        .join("hal")
        .join("ral");

    // Apply any patches to the SVD.
    info!("applying patches to SVD file");

    let yaml_file = svd_path.join("patches").join("ieee802154.yml");
    process_file(&yaml_file)?;

    let from = svd_path.join("ieee802154.base.svd.patched");
    let to = svd_path.join("ieee802154.svd");
    fs::rename(from, to)?;

    info!(
        "generating register access layer from '{}'",
        svd_file.display()
    );

    let config = Config {
        target: Target::RISCV,
        output_dir: out_dir.clone(),
        const_generic: true,

        ..Config::default()
    };

    let input = fs::read_to_string(svd_file)?;
    let device = load_from(&input, &config)?;

    let mut device_x = String::new();
    let items = render(&device, &config, &mut device_x)?;
    let data = items.to_string()
        .replace("crate :: ", "crate :: hal :: ral :: ")
        .replace(
            "# ! [deny (dead_code)] # ! [deny (improper_ctypes)] # ! [deny (missing_docs)] # ! [deny (no_mangle_generic_items)] # ! [deny (non_shorthand_field_patterns)] # ! [deny (overflowing_literals)] # ! [deny (path_statements)] # ! [deny (patterns_in_fns_without_body)] # ! [deny (private_in_public)] # ! [deny (unconditional_recursion)] # ! [deny (unused_allocation)] # ! [deny (unused_comparisons)] # ! [deny (unused_parens)] # ! [deny (while_true)] # ! [allow (non_camel_case_types)] # ! [allow (non_snake_case)] # ! [no_std]",
            "# ! [allow (non_camel_case_types)] # ! [allow (non_snake_case)] #![allow(unused)]"
        )
        .replace("DEVICE_PERIPHERALS", "IEEE802154_PERIPHERALS");

    let mut file = File::create(out_dir.join(&format!("{}.rs", chip.to_string())))?;
    file.write_all(data.as_ref())?;

    let out_dir = out_dir
        .as_os_str()
        .to_str()
        .unwrap()
        .to_string()
        .replace("\\\\?\\", "");

    let chip_name = chip.to_string();
    let file_name = format!("{out_dir}/{chip_name}.rs");

    info!("formatting source file '{file_name}'");
    Command::new("rustfmt").arg(file_name).status()?;

    info!("All done.");

    Ok(())
}

fn generate_binary_includes(workspace: &Path, chip: Chip) -> Result<()> {
    let base_dirs = BaseDirs::new().unwrap();
    let home_path = base_dirs.home_dir();
    let ieee_path = workspace.join("esp-ieee802154");

    info!("generating bindings with bindgen");
    let bindings = Builder::default()
        .clang_args([
            format!("-I{}", ieee_path.join("headers").display()),
            format!("-I{}", ieee_path.join("include").display()),
            format!(
                "-I{}",
                home_path
                    .join(".espressif")
                    .join("tools")
                    .join("riscv32-esp-elf")
                    .join("esp-12.2.0_20230208")
                    .join("riscv32-esp-elf")
                    .join("riscv32-esp-elf")
                    .join("include")
                    .display()
            ),
            format!("-DCONFIG_IDF_TARGET_{}", chip.to_string().to_uppercase()),
            format!("-DCONFIG_SOC_IEEE802154_SUPPORTED=y"),
        ])
        .ctypes_prefix("crate::binary::c_types")
        .derive_debug(false)
        .header(
            ieee_path
                .join("include")
                .join("include.h")
                .to_string_lossy(),
        )
        .layout_tests(false)
        .raw_line("#![allow(dead_code)]")
        .raw_line("#![allow(improper_ctypes)]")
        .raw_line("#![allow(non_camel_case_types)]")
        .raw_line("#![allow(non_snake_case)]")
        .raw_line("#![allow(non_upper_case_globals)]")
        .use_core()
        .generate()?;

    let bindings_path = ieee_path
        .join("src")
        .join("binary")
        .join("include")
        .join(format!("{chip}.rs"));

    bindings.write_to_file(&bindings_path)?;
    info!(
        "generated bindings written to '{}'",
        bindings_path.display()
    );

    Ok(())
}
