use std::{
    env,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::Result;
use bindgen::Builder;
use clap::{Parser, Subcommand, ValueEnum};
use directories::BaseDirs;
use log::{info, LevelFilter};
use strum::Display;
use svd2rust::{generate::device::render, load_from, Config, Target};
use svdtools::patch::process_file;
use xshell::{cmd, Shell};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, ValueEnum)]
#[strum(serialize_all = "lowercase")]
enum Chip {
    /// ESP32-H4
    Esp32h4,
}

#[derive(Debug, Parser)]
struct Cli {
    #[clap(subcommand)]
    subcommand: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Generate the Register Access Layer (RAL)
    Ral,
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

    let args = Cli::parse().subcommand;

    // The directory containing the cargo manifest for the 'xtask' package is a
    // subdirectory within the cargo workspace.
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace = workspace.parent().unwrap().canonicalize()?;

    match args {
        Commands::Ral => generate_register_access_layer(&workspace),
        Commands::Includes { chip } => generate_binary_includes(&workspace, chip),
    }
}

fn generate_register_access_layer(workspace: &Path) -> Result<()> {
    let svd_path = workspace.join("svd");
    let svd_file = svd_path.join("ieee802154.svd");
    let out_dir = workspace.join("esp-ieee802154").join("src").join("ral");

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
        .replace("crate :: ", "crate :: ral :: ")
        .replace(
            "# ! [deny (dead_code)] # ! [deny (improper_ctypes)] # ! [deny (missing_docs)] # ! [deny (no_mangle_generic_items)] # ! [deny (non_shorthand_field_patterns)] # ! [deny (overflowing_literals)] # ! [deny (path_statements)] # ! [deny (patterns_in_fns_without_body)] # ! [deny (private_in_public)] # ! [deny (unconditional_recursion)] # ! [deny (unused_allocation)] # ! [deny (unused_comparisons)] # ! [deny (unused_parens)] # ! [deny (while_true)] # ! [allow (non_camel_case_types)] # ! [allow (non_snake_case)] # ! [no_std]",
            "# ! [allow (non_camel_case_types)] # ! [allow (non_snake_case)] #![allow(unused)]"
        )
        .replace("DEVICE_PERIPHERALS", "IEEE802154_PERIPHERALS");

    let mut file = File::create(out_dir.join("mod.rs"))?;
    file.write_all(data.as_ref())?;

    let out_dir = out_dir
        .as_os_str()
        .to_str()
        .unwrap()
        .to_string()
        .replace("\\\\?\\", "");

    let sh = Shell::new()?;
    cmd!(sh, "rustfmt {out_dir}/mod.rs").quiet().run()?;

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
            format!(
                "-I{}",
                ieee_path.join("headers").join(chip.to_string()).display()
            ),
            format!("-I{}", ieee_path.join("include").display()),
            format!(
                "-I{}",
                home_path
                    .join(".espressif")
                    .join("tools")
                    .join("riscv32-esp-elf")
                    .join("esp-2021r2-8.4.0")
                    .join("riscv32-esp-elf")
                    .join("riscv32-esp-elf")
                    .join("include")
                    .display()
            ),
            format!("-DCONFIG_IDF_TARGET_{}", chip.to_string().to_uppercase()),
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
