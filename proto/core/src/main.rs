use crate::log::Logger;
use crate::runtime::Runtime;
use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use ::log::{info, LevelFilter};
use ves_art_core::sprite::Tile;

mod log;
mod runtime;

struct ProtoCore {
    logger: Logger,
    vrom: Vrom,
}

impl ProtoCore {
    fn new(wasm_file: impl AsRef<Path>) -> Result<ProtoCore> {
        // Vrom
        let vrom = Vrom::from_file(&wasm_file)?;

        // Logger
        let wasm_filename = wasm_file
            .as_ref()
            .file_name()
            .ok_or_else(|| anyhow!("No file name found on path: {:?}", wasm_file.as_ref()))?
            .to_str()
            .ok_or_else(|| {
                anyhow!(
                    "Could not convert file name to string: {:?}",
                    wasm_file.as_ref()
                )
            })?;
        let mut log_dir = PathBuf::from("/tmp/ves_proto");
        std::fs::create_dir_all(&log_dir)?;
        log_dir.push(wasm_filename);
        let logger = Logger::from_file(log_dir)?;

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
        .with_module_level("ves_proto_core", LevelFilter::Info)
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
