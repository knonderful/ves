use std::path::{Path, PathBuf};
use std::time::Duration;

use ::log::{info, LevelFilter};
use anyhow::{anyhow, Result};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::surface::Surface;

use ves_art_core::sprite::Tile;
use ves_proto_common::gpu::{
    OamTableEntry, OamTableIndex, PaletteColor, PaletteIndex, PaletteTableIndex,
};

use crate::log::Logger;
use crate::runtime::Runtime;

mod log;
mod runtime;

struct ProtoCore {
    logger: Logger,
    vrom: Vrom,
    oam: [OamTableEntry; 128],
    palettes: [Palette; 256],
}

#[derive(Copy, Clone, Debug, Default)]
struct Palette {
    colors: [PaletteColor; 16], // 1st entry is transparent
}

impl ProtoCore {
    fn new(wasm_file: impl AsRef<Path>) -> Result<ProtoCore> {
        let vrom = Vrom::from_file(&wasm_file)?;
        let logger = Logger::new();

        Ok(Self {
            logger,
            vrom,
            oam: [Default::default(); 128],
            palettes: [Default::default(); 256],
        })
    }

    pub(crate) fn set_oam_entry(&mut self, index: OamTableIndex, entry: OamTableEntry) {
        self.oam[usize::from(index)] = entry;
    }

    pub(crate) fn set_palette_entry(
        &mut self,
        palette: PaletteTableIndex,
        index: PaletteIndex,
        color: PaletteColor,
    ) {
        let palette = &mut self.palettes[usize::from(palette)];
        palette.colors[usize::from(index)] = color;
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
        .window("SDL2", 512, 448)
        .position_centered()
        .build()?;

    info!("Creating canvas.");
    let mut canvas = window.into_canvas().build()?;

    info!("Starting game loop.");
    let mut event_pump = sdl_context
        .event_pump()
        .map_err(|e| anyhow!("Could not initialize SDL: {}", e))?;

    let texture_creator = canvas.texture_creator();
    info!(
        "Canvas default pixel format: {:?}",
        &canvas.default_pixel_format()
    );

    let mut running = true;
    while running {
        // Advance game state
        let core = runtime.step(instance_ptr)?;

        // Event handling
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

        // Create temporary surface to render our scene onto
        let mut target =
            sdl2::surface::Surface::new(256, 224, sdl2::pixels::PixelFormatEnum::RGBA32)
                .map_err(|err| anyhow!("Could not create target surface: {err}"))?;

        // Render the scene
        render_oam(&mut target, &core.oam, &core.palettes, &core.vrom)?;

        // Create a texture for the scene surface
        let texture = texture_creator.create_texture_from_surface(&target)?;

        // Render onto the window canvas
        canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 64));
        canvas.clear();
        canvas
            .copy(&texture, None, None)
            .map_err(|err| anyhow!("Could not copy texture onto window canvas: {err}"))?;
        canvas.present();

        // Slow down (for now)
        std::thread::sleep(Duration::from_millis(100));
    }

    Ok(())
}

fn render_oam(
    target: &mut Surface,
    oam: &[OamTableEntry],
    palettes: &[Palette],
    vrom: &Vrom,
) -> Result<()> {
    for obj in oam.iter().rev() {
        let char_table_index = usize::try_from(obj.char_table_index())
            .map_err(|_| anyhow!("Could not convert char_table_index to usize."))?;
        let tile = &vrom.tiles[char_table_index];

        let palette = &palettes[usize::from(obj.palette_table_index())];
        let surface = create_sdl_surface(tile, palette, obj.h_flip(), obj.v_flip())?;

        use ves_art_core::surface::Surface as _;
        let dest_rect = sdl2::rect::Rect::new(
            obj.position().0.into(),
            obj.position().1.into(),
            tile.surface().size().width.raw(),
            tile.surface().size().height.raw(),
        );

        surface
            .blit(None, target, dest_rect)
            .map_err(|err| anyhow!("Could not blit surface onto target surface: {err}"))?;
    }
    Ok(())
}

fn create_sdl_surface(
    tile: &Tile,
    palette: &Palette,
    hflip: bool,
    vflip: bool,
) -> Result<sdl2::surface::Surface<'static>> {
    use ves_art_core::surface::Surface as _;
    let surf = tile.surface();
    let size = surf.size();
    let width = size.width.raw();
    let height = size.height.raw();

    // NOTE: Using RGBA32 and not RGBA8888, since that gives us a platform-indepenent lay-out in
    //       memory.
    let mut out_surface =
        sdl2::surface::Surface::new(width, height, sdl2::pixels::PixelFormatEnum::RGBA32)
            .map_err(|err| anyhow!("Could not create surface: {err}"))?;
    let mut dest_iter = out_surface
        .without_lock_mut() // we just created the surface, so we know it's a software surface
        .ok_or_else(|| anyhow!("Could not lock surface data."))?
        .iter_mut();

    let src_data = surf.data();
    ves_art_core::surface::surface_iterate(size, size.as_rect(), hflip, vflip, |_, src_idx| {
        let pal_idx: usize = src_data[src_idx].value().into();
        let (r, g, b) = palette.colors[pal_idx].to_real();
        // The first entry in the palette is reserved for transparency
        let a = if pal_idx == 0 {
            0
        } else {
            255
        };
        *dest_iter.next().unwrap() = r;
        *dest_iter.next().unwrap() = g;
        *dest_iter.next().unwrap() = b;
        *dest_iter.next().unwrap() = a;
    })
    .map_err(|err| anyhow!("Could not generate output surface data: {err}"))?;

    Ok(out_surface)
}
