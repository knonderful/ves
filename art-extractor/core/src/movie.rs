use crate::{Palette, Size, Sprite, Tile};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FrameRate {
    Ntsc,
    Pal,
}

impl FrameRate {
    /// Retrieves the number of frames per second.
    pub fn fps(&self) -> usize {
        match self {
            FrameRate::Ntsc => 60,
            FrameRate::Pal => 50,
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MovieFrame {
    frame_number: usize,
    sprites: Vec<Sprite>,
}

impl MovieFrame {
    /// Creates a new instance.
    pub fn new(frame_number: usize, sprites: Vec<Sprite>) -> Self {
        Self { frame_number, sprites }
    }
}
