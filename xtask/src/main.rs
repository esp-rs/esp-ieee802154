use std::{
    env,
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use anyhow::Result;
use clap::Parser;
use log::{info, LevelFilter};
use svd2rust::{generate::device::render, load_from, Config, Target};
use svdtools::patch::process_file;
use xshell::{cmd, Shell};

#[derive(Debug, Parser)]
struct Opts {}

fn main() -> Result<()> {
    env_logger::Builder::new()
        .filter_module("xtask", LevelFilter::Info)
        .init();

    let _opts = Opts::parse();

    // The directory containing the cargo manifest for the 'xtask' package is a
    // subdirectory within the cargo workspace.
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace = workspace.parent().unwrap().canonicalize()?;

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
