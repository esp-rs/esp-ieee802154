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

    let svd_file = workspace.join("svd/ieee802154.svd");
    let out_dir = workspace.join("esp-ieee802154/src/ral");
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
    let data = items.to_string();

    let mut file = File::create(out_dir.join("mod.rs"))?;
    file.write_all(data.as_ref())?;

    info!("All done. Manually format ./esp-ieee802154/src/ral/mod.rs now and replace `crate::` with `crate::ral::`.\nRemove and add allo/deny attributes as needed.");

    Ok(())
}
