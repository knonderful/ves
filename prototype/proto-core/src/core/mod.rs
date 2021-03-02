pub mod geometry;

use libretro_backend::{AudioVideoInfo, CoreInfo, GameData, LoadGameResult, RuntimeHandle};

use crate::game::Game;
use std::path::Path;
use crate::core::geometry::{Dimensions, Rectangle, CoordinateType};

const SCREEN_WIDTH: u32 = 320;
const SCREEN_HEIGHT: u32 = 240;

trait Pixel: From<(u8, u8, u8)> + From<(u8, u8, u8, u8)> {
    fn from_rgb(r: u8, g: u8, b: u8) -> Self;
    fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self;
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PixelArgb8888 {
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

pub type FrameBufferPixel = PixelArgb8888;

pub struct FrameBuffer {
    dimensions: Dimensions,
    data: Vec<FrameBufferPixel>,
}

impl FrameBuffer {
    pub fn new(dimensions: Dimensions) -> Self {
        FrameBuffer {
            dimensions,
            data: vec!(Default::default(); dimensions.area() as usize),
        }
    }

    pub fn rectangle(&mut self, rectangle: Rectangle) -> FrameBufferRectangle {
        FrameBufferRectangle::new(self, rectangle)
    }

    pub fn width(&self) -> CoordinateType {
        self.dimensions.width
    }

    #[allow(dead_code)]
    pub fn height(&self) -> CoordinateType {
        self.dimensions.height
    }
}

pub struct FrameBufferRectangle<'fb> {
    frame_buffer: &'fb mut FrameBuffer,
    rectangle: Rectangle,
}

impl<'fb> FrameBufferRectangle<'fb> {
    fn new(frame_buffer: &'fb mut FrameBuffer, rectangle: Rectangle) -> Self {
        Self { frame_buffer, rectangle }
    }

    pub fn write(&mut self, data: &[FrameBufferPixel]) {
        assert_eq!(self.rectangle.area(), data.len() as CoordinateType);
        let origin = self.rectangle.origin;
        let pitch = self.frame_buffer.width() as usize;
        let mut dest_index = origin.y as usize * pitch + origin.x as usize;
        let mut src_index = 0;
        let rect_width = self.rectangle.width() as usize;
        for _ in self.rectangle.y_range() {
            self.frame_buffer.data[dest_index..dest_index + rect_width].copy_from_slice(&data[src_index..src_index + rect_width]);
            src_index += rect_width;
            dest_index += pitch;
        }
    }
}

pub struct ProtoCore {
    game: Option<Game>,
    game_data: Option<GameData>,
    frame_buffer: FrameBuffer,
    frame_count: u64,
}

impl ProtoCore {
    pub fn new() -> Self {
        ProtoCore {
            game: Option::None,
            game_data: Option::None,
            frame_buffer: FrameBuffer::new(Dimensions::new(SCREEN_WIDTH, SCREEN_HEIGHT)),
            frame_count: 0,
        }
    }
}

impl Default for ProtoCore {
    fn default() -> Self {
        ProtoCore::new()
    }
}

impl libretro_backend::Core for ProtoCore {
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
