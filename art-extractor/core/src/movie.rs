use crate::{Palette, Size, Sprite, Tile};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Movie {
    screen_size: Size,
    palettes: Vec<Palette>,
    tiles: Vec<Tile>,
    frames: Vec<MovieFrame>,
}

impl Movie {
    /// Creates a new instance.
    pub fn new(screen_size: Size, palettes: Vec<Palette>, tiles: Vec<Tile>, frames: Vec<MovieFrame>) -> Self {
        Self { screen_size, palettes, tiles, frames }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MovieFrame {
    sprites: Vec<Sprite>,
}

impl MovieFrame {
    /// Creates a new instance.
    pub fn new(sprites: Vec<Sprite>) -> Self {
        Self { sprites }
    }
}
