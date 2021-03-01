mod game_api;
mod game;

#[macro_use]
extern crate libretro_backend;

use libretro_backend::{AudioVideoInfo, CoreInfo, GameData, LoadGameResult, RuntimeHandle};

use crate::game::Game;
use std::path::Path;

const SCREEN_WIDTH: u32 = 320;
const SCREEN_HEIGHT: u32 = 240;

trait Pixel: From<(u8, u8, u8)> + From<(u8, u8, u8, u8)> {
    fn from_rgb(r: u8, g: u8, b: u8) -> Self;
    fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self;
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
struct PixelArgb8888 {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Pixel for PixelArgb8888 {
    fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self::from_rgba(r, g, b, 255)
    }

    fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

impl From<(u8, u8, u8)> for PixelArgb8888 {
    fn from(components: (u8, u8, u8)) -> Self {
        Self::from_rgb(components.0, components.1, components.2)
    }
}

impl From<(u8, u8, u8, u8)> for PixelArgb8888 {
    fn from(components: (u8, u8, u8, u8)) -> Self {
        Self::from_rgba(components.0, components.1, components.2, components.3)
    }
}

impl Default for PixelArgb8888 {
    fn default() -> Self {
        Self::from_rgb(0, 0, 0)
    }
}

struct FrameBufferCursor<'a> {
    frame_buffer: &'a mut FrameBuffer,
    offset: usize,
}

impl<'a> FrameBufferCursor<'a> {
    fn new(frame_buffer: &'a mut FrameBuffer) -> Self {
        FrameBufferCursor { frame_buffer, offset: 0 }
    }

    pub fn move_to(&mut self, x: u32, y: u32) {
        let offset = (y * self.frame_buffer.width) as usize + x as usize;
        assert!(offset < self.frame_buffer.data.len());
        self.offset = offset;
    }

    pub fn set_pixel(&mut self, pixel: FrameBufferPixel) {
        self.frame_buffer.data[self.offset] = pixel;
    }

    pub fn advance(&mut self) {
        self.offset += 1;
    }
}

type FrameBufferPixel = PixelArgb8888;

struct FrameBuffer {
    width: u32,
    _height: u32,
    data: Vec<FrameBufferPixel>,
}

impl FrameBuffer {
    pub fn new(width: u32, height: u32) -> Self {
        FrameBuffer {
            width,
            _height: height,
            data: vec!(Default::default(); (width * height) as usize),
        }
    }

    pub fn cursor(&mut self) -> FrameBufferCursor {
        FrameBufferCursor::new(self)
    }
}

struct Emulator {
    game: Option<Game>,
    game_data: Option<GameData>,
    frame_buffer: FrameBuffer,
    frame_count: u64,
}

impl Emulator {
    pub fn new() -> Self {
        Emulator {
            game: Option::None,
            game_data: Option::None,
            frame_buffer: FrameBuffer::new(SCREEN_WIDTH, SCREEN_HEIGHT),
            frame_count: 0,
        }
    }
}

impl Default for Emulator {
    fn default() -> Self {
        Emulator::new()
    }
}

impl libretro_backend::Core for Emulator {
    fn info() -> CoreInfo {
        CoreInfo::new("ves-proto", "0.0.1")
            .supports_roms_with_extension("wasm")
    }

    fn on_load_game(&mut self, game_data: GameData) -> LoadGameResult {
        if let Some(path) = game_data.path() {
            if let Ok(game) = Game::from_path(Path::new(path)) {
                self.game.replace(game);
                self.game_data.replace(game_data);
                let av_info = AudioVideoInfo::new()
                    .video(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32, 60.0, libretro_backend::PixelFormat::ARGB8888);
                return LoadGameResult::Success(av_info);
            }
        }

        LoadGameResult::Failed(game_data)
    }

    fn on_unload_game(&mut self) -> GameData {
        self.game_data.take().unwrap()
    }

    fn on_run(&mut self, handle: &mut RuntimeHandle) {
        self.frame_count += 1;

        let game = self.game.as_ref().unwrap();
        game.step();
        game.render(&mut self.frame_buffer);

        let video_data = unsafe {
            // We are in control of FrameBufferPixel. It is aligned in ARGB8888 format.
            // Defining the frame buffer data in this type (instead of u8) is more ergonomic
            // everywhere in our code, but it does require this cast to be efficient.
            std::mem::transmute::<&[FrameBufferPixel], &[u8]>(self.frame_buffer.data.as_slice())
        };
        handle.upload_video_frame(video_data);

        // No audio for now, but we need to call this
        handle.upload_audio_frame(&[]);
    }

    fn on_reset(&mut self) {
        // TODO: Handle reset.
    }
}

libretro_core!( Emulator );