use crate::{Palette, Size, Sprite, Tile};

#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FrameRate {
    Ntsc,
    Pal,
}

impl FrameRate {
    /// Retrieves the number of frames per second.
    pub fn fps(&self) -> u64 {
        match self {
            FrameRate::Ntsc => 60,
            FrameRate::Pal => 50,
        }
    }
}

#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Movie {
    screen_size: Size,
    palettes: Vec<Palette>,
    tiles: Vec<Tile>,
    frames: Vec<MovieFrame>,
    frame_rate: FrameRate,
}

impl Movie {
    /// Creates a new instance.
    pub fn new(screen_size: Size, palettes: Vec<Palette>, tiles: Vec<Tile>, frames: Vec<MovieFrame>, frame_rate: FrameRate) -> Self {
        Self { screen_size, palettes, tiles, frames, frame_rate }
    }

    /// Retrieves the screen size.
    pub fn screen_size(&self) -> Size {
        self.screen_size
    }

    /// Retrieves the palettes.
    pub fn palettes(&self) -> &[Palette] {
        &self.palettes
    }

    /// Retrieves the tiles.
    pub fn tiles(&self) -> &[Tile] {
        &self.tiles
    }

    /// Retrieves the frames.
    pub fn frames(&self) -> &[MovieFrame] {
        &self.frames
    }

    /// Retrieves the frame rate.
    pub fn frame_rate(&self) -> FrameRate {
        self.frame_rate
    }
}

#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MovieFrame {
    frame_number: u64,
    sprites: Vec<Sprite>,
}

impl MovieFrame {
    /// Creates a new instance.
    pub fn new(frame_number: u64, sprites: Vec<Sprite>) -> Self {
        Self { frame_number, sprites }
    }

    /// Retrieves the frame number.
    pub fn frame_number(&self) -> u64 {
        self.frame_number
    }

    /// Retrieves the sprites.
    pub fn sprites(&self) -> &[Sprite] {
        &self.sprites
    }
}
