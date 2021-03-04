pub mod geometry;

use libretro_backend::{AudioVideoInfo, CoreInfo, GameData, LoadGameResult, RuntimeHandle};

use crate::game::Game;
use std::path::Path;
use crate::core::geometry::{Dimensions, Rectangle, CoordinateType, Position};

const SCREEN_WIDTH: u32 = 320;
const SCREEN_HEIGHT: u32 = 240;

pub trait Pixel: From<(u8, u8, u8)> + From<(u8, u8, u8, u8)> {
    fn from_rgb(r: u8, g: u8, b: u8) -> Self;
    fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self;

    fn set_rgb(&mut self, r: u8, g: u8, b: u8);
    fn set_rgba(&mut self, r: u8, g: u8, b: u8, a: u8);
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PixelArgb8888 {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

const ALPHA_OPAQUE: u8 = 255;

impl Pixel for PixelArgb8888 {
    fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self::from_rgba(r, g, b, ALPHA_OPAQUE)
    }

    fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    fn set_rgb(&mut self, r: u8, g: u8, b: u8) {
        self.set_rgba(r, g, b, ALPHA_OPAQUE);
    }

    fn set_rgba(&mut self, r: u8, g: u8, b: u8, a: u8) {
        self.r = r;
        self.g = g;
        self.b = b;
        self.a = a;
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

    pub fn window(&mut self, rectangle: Rectangle) -> FrameBufferWindow {
        FrameBufferWindow::new(self, rectangle)
    }

    pub fn width(&self) -> CoordinateType {
        self.dimensions.width
    }

    #[allow(dead_code)]
    pub fn height(&self) -> CoordinateType {
        self.dimensions.height
    }
}

pub struct FrameBufferWindow<'fb> {
    framebuffer: &'fb mut FrameBuffer,
    origin_x: CoordinateType,
    position: Position,
    final_position: Position,
}

impl<'fb> FrameBufferWindow<'fb> {
    fn new(framebuffer: &'fb mut FrameBuffer, rectangle: Rectangle) -> Self {
        // The underlying type from the euclid crate takes the maximum position as _inclusive_ (probably
        // because it has to be able to work with float-based coordinates, in which case that makes sense).
        // For our case that doesn't make sense, so we deduct 1 from both coordinate elements.
        let final_position = (rectangle.max() - Position::new(1, 1)).to_point();
        let position = rectangle.origin;

        assert!(final_position.x < framebuffer.width());
        assert!(final_position.y < framebuffer.height());

        Self {
            framebuffer,
            origin_x: position.x,
            position,
            final_position,
        }
    }
}

impl<'fb> Iterator for FrameBufferWindow<'fb> {
    type Item = &'fb mut FrameBufferPixel;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position.y > self.final_position.y {
            return None;
        }

        let pitch = self.framebuffer.width() as usize;
        let index = self.position.y as usize * pitch + self.position.x as usize;

        // We need to "reset" the lifetime in order to get to the 'fb lifetime.
        // This is safe because the constructor guarantees that the rectangle is within the data buffer.
        let pixel = unsafe {
            let x = self.framebuffer.data.as_mut_ptr().add(index);
            &mut *x
        };

        if self.position.x == self.final_position.x {
            self.position.x = self.origin_x;
            self.position.y += 1;
        } else {
            self.position.x += 1;
        }

        Some(pixel)
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
