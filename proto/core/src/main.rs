use std::path::{Path, PathBuf};

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

/// The width of the visible screen area in pixels.
const SCREEN_VISIBLE_WIDTH: u32 = 256;
/// The height of the visible screen area in pixels.
const SCREEN_VISIBLE_HEIGHT: u32 = 224;

/// The width of the screen buffer in pixels.
const SCREEN_BUFFER_WIDTH: u32 = 512;
/// The height of the screen buffer in pixels.
const SCREEN_BUFFER_HEIGHT: u32 = 256;

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
        .window("SDL2", SCREEN_VISIBLE_WIDTH * 2, SCREEN_VISIBLE_HEIGHT * 2)
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

    let mut fps_manager = sdl2::gfx::framerate::FPSManager::new();
    fps_manager
        .set_framerate(60)
        .map_err(|err| anyhow!("Can not set framerate: {err}"))?;

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
        // NOTE: Using RGBA32 and not RGBA8888, since that gives us a platform-indepenent lay-out in
        //       memory.
        let mut target = sdl2::surface::Surface::new(
            SCREEN_BUFFER_WIDTH,
            SCREEN_BUFFER_HEIGHT,
            sdl2::pixels::PixelFormatEnum::RGBA32,
        )
        .map_err(|err| anyhow!("Could not create target surface: {err}"))?;

        // Render the scene
        render_oam(&mut target, &core.oam, &core.palettes, &core.vrom)?;

        // Create a texture for the scene surface
        let texture = texture_creator.create_texture_from_surface(&target)?;

        // Render onto the window canvas
        canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 64));
        canvas.clear();
        canvas
            .copy(
                &texture,
                sdl2::rect::Rect::new(0, 0, SCREEN_VISIBLE_WIDTH, SCREEN_VISIBLE_HEIGHT),
                None,
            )
            .map_err(|err| anyhow!("Could not copy texture onto window canvas: {err}"))?;
        canvas.present();

        fps_manager.delay();
    }

    Ok(())
}

fn render_oam(
    screen_buffer: &mut Surface,
    oam: &[OamTableEntry],
    palettes: &[Palette],
    vrom: &Vrom,
) -> Result<()> {
    for obj in oam.iter().rev() {
        let char_table_index = usize::try_from(obj.char_table_index())
            .map_err(|_| anyhow!("Could not convert char_table_index to usize."))?;
        let tile = &vrom.tiles[char_table_index];

        let palette = &palettes[usize::from(obj.palette_table_index())];
        render_tile(
            screen_buffer,
            tile,
            palette,
            obj.position(),
            obj.h_flip(),
            obj.v_flip(),
        )?;
    }
    Ok(())
}

fn render_tile(
    screen_buffer: &mut Surface,
    tile: &Tile,
    palette: &Palette,
    position: (u16, u16),
    hflip: bool,
    vflip: bool,
) -> Result<()> {
    // Checking some presumptions about the calling code
    debug_assert!(!screen_buffer.must_lock());
    debug_assert_eq!(
        screen_buffer.pixel_format_enum(),
        sdl2::pixels::PixelFormatEnum::RGBA32
    );

    use ves_art_core::surface::Surface as _;
    let surf = tile.surface();
    let src_size = surf.size();
    let src_data = surf.data();

    let dest_data = screen_buffer
        .without_lock_mut()
        .ok_or_else(|| anyhow!("Could not lock surface data."))?;

    ves_art_core::surface::surface_iterate_2(
        src_size,
        src_size.as_rect(),
        ves_art_core::geom_art::Size::new(SCREEN_BUFFER_WIDTH, SCREEN_BUFFER_HEIGHT),
        ves_art_core::geom_art::Point::new(u32::from(position.0), u32::from(position.1)),
        hflip,
        vflip,
        |_, src_idx, _, dest_idx| {
            // Get the index in the palette
            let pal_idx: usize = src_data[src_idx].value().into();
            // The first entry in the palette is reserved for transparency (aka: write nothing)
            if pal_idx == 0 {
                return;
            }
            // Get the color value
            let (r, g, b) = palette.colors[pal_idx].to_real();

            // Write the color to the target surface
            let i = 4 * dest_idx; // because RGBA32 is 4 bytes per pixel
            dest_data[i] = r;
            dest_data[i + 1] = g;
            dest_data[i + 2] = b;
            dest_data[i + 3] = 255;
        },
    )
    .map_err(|err| anyhow!("Could not render object onto screen buffer: {err}"))?;

    Ok(())
}
