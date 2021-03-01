mod game_api;
mod game;

#[macro_use]
extern crate libretro_backend;

use libretro_backend::{AudioVideoInfo, CoreInfo, GameData, LoadGameResult, PixelFormat, RuntimeHandle};

use crate::game::Game;
use std::path::Path;

const SCREEN_WIDTH: u32 = 320;
const SCREEN_HEIGHT: u32 = 240;
const FRAME_BUFFER_BYTES_PER_PIXEL: usize = 4;

#[derive(Debug, Copy, Clone, PartialEq)]
struct FrameBufferColor {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl FrameBufferColor {
    pub fn red(&self) -> u8 {
        self.r
    }

    pub fn green(&self) -> u8 {
        self.g
    }

    pub fn blue(&self) -> u8 {
        self.b
    }

    pub fn alpha(&self) -> u8 {
        self.a
    }
}

impl From<(u8, u8, u8)> for FrameBufferColor {
    fn from(values: (u8, u8, u8)) -> Self {
        FrameBufferColor { r: values.0, g: values.1, b: values.2, a: 255 }
    }
}

impl From<(u8, u8, u8, u8)> for FrameBufferColor {
    fn from(values: (u8, u8, u8, u8)) -> Self {
        FrameBufferColor { r: values.0, g: values.1, b: values.2, a: values.3 }
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
        let offset = (y * self.frame_buffer.width) as usize * FRAME_BUFFER_BYTES_PER_PIXEL + x as usize * FRAME_BUFFER_BYTES_PER_PIXEL;
        assert!(offset < self.frame_buffer.data.len());
        self.offset = offset;
    }

    pub fn draw_color(&mut self, color: FrameBufferColor) {
        match color.alpha() {
            0 => {},
            255 => {
                self.frame_buffer.data[self.offset] = color.blue();
                self.frame_buffer.data[self.offset + 1] = color.green();
                self.frame_buffer.data[self.offset + 2] = color.red();
            },
            _ => unimplemented!("Blending not supported yet."),
        }

        self.offset += FRAME_BUFFER_BYTES_PER_PIXEL;
    }
}

struct FrameBuffer {
    width: u32,
    _height: u32,
    data: Vec<u8>,
}

impl FrameBuffer {
    pub fn new(width: u32, height: u32) -> Self {
        FrameBuffer {
            width,
            _height: height,
            data: vec!(0; (width * height) as usize * FRAME_BUFFER_BYTES_PER_PIXEL),
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
                    .video(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32, 60.0, PixelFormat::ARGB8888);
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

        handle.upload_video_frame(self.frame_buffer.data.as_slice());

        // No audio for now, but we need to call this
        handle.upload_audio_frame(&[]);
    }

    fn on_reset(&mut self) {
        // TODO: Handle reset.
    }
}

libretro_core!( Emulator );