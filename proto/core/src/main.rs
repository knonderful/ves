use std::path::{Path, PathBuf};

use ::log::{info, LevelFilter};
use anyhow::{anyhow, Result};

use ves_art_core::sprite::Tile;

use crate::log::Logger;
use crate::runtime::Runtime;

mod log;
mod runtime;

struct ProtoCore {
    logger: Logger,
    vrom: Vrom,
}

impl ProtoCore {
    fn new(wasm_file: impl AsRef<Path>) -> Result<ProtoCore> {
        let vrom = Vrom::from_file(&wasm_file)?;
        let logger = Logger::new();

        Ok(Self { logger, vrom })
    }
}

struct Vrom {
    tiles: Vec<Tile>,
}

impl Vrom {
    pub fn from_file(wasm_file: impl AsRef<Path>) -> Result<Vrom> {
        const SECTION_NAME: &str = "vrom";

        let module = parity_wasm::deserialize_file(&wasm_file)?;
        let payload = module
            .custom_sections()
            .find(|sect| sect.name() == SECTION_NAME)
            .ok_or_else(|| {
                anyhow::Error::msg(format!(
                    "Could not find rom data (custom section '{}') in {}.",
                    SECTION_NAME,
                    wasm_file.as_ref().display()
                ))
            })?
            .payload();

        Self::from_bincode(payload)
    }

    fn from_bincode(data: &[u8]) -> Result<Vrom> {
        let tiles: Vec<Tile> = bincode::deserialize_from(data)?;

        info!("VROM summary:");
        info!("  {} tiles", tiles.len());

        Ok(Self { tiles })
    }
}

fn main() -> Result<()> {
    simple_logger::SimpleLogger::new()
        .with_level(LevelFilter::Off)
        .with_module_level(env!("CARGO_CRATE_NAME"), LevelFilter::Info)
        .init()?;

    let args: Vec<String> = std::env::args().collect();
    let wasm_file = PathBuf::from(&args[1]).canonicalize()?;
    info!("Running core.");
    info!(
        "Loading WASM file: {}",
        wasm_file
            .as_path()
            .to_str()
            .ok_or_else(|| anyhow!("The provided path can not be converted to a string."))?
    );

    let wasm_file = wasm_file.as_path();
    let core = ProtoCore::new(wasm_file)?;
    let mut runtime = Runtime::from_path(wasm_file, core)?;
    info!("Creating game instance.");
    let instance_ptr = runtime.create_instance()?;

    info!("Starting game loop.");
    // TODO: Implement actual game loop with SDL
    runtime.step(instance_ptr)?;
    runtime.step(instance_ptr)?;
    runtime.step(instance_ptr)?;
    runtime.step(instance_ptr)?;
    runtime.step(instance_ptr)?;

    Ok(())
}
