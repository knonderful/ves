use std::path::{Path, PathBuf};
use std::time::Duration;

use ::log::{info, LevelFilter};
use anyhow::{anyhow, Result};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

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

    info!("Initializing SDL.");
    let sdl_context = sdl2::init().map_err(|e| anyhow!("Could not initialize SDL: {}", e))?;
    let video_subsystem = sdl_context
        .video()
        .map_err(|e| anyhow!("Could not initialize SDL: {}", e))?;

    info!("Initializing video subsystem.");
    let window = video_subsystem
        .window("SDL2", 640, 480)
        .position_centered()
        .build()?;

    info!("Creating canvas.");
    let mut canvas = window.into_canvas().accelerated().build()?;

    info!("Starting game loop.");
    let mut event_pump = sdl_context
        .event_pump()
        .map_err(|e| anyhow!("Could not initialize SDL: {}", e))?;

    let mut running = true;
    while running {
        runtime.step(instance_ptr)?;

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    running = false;
                }
                _ => {}
            }
        }

        canvas.clear();
        canvas.present();

        std::thread::sleep(Duration::from_millis(100));
    }

    Ok(())
}
