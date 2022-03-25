//! A module for artwork-related data (not only sprites, despite being called `sprite`).
//!
//! The main components are:
//! * [`Tile`]: The smallest graphical element.
//! * [`Sprite`]: A tile with a position and some additional flags like horizontal and vertical flipping.
//! * [`Cel`]: A composition of `Sprite`s.
//! * [`Animation`]: A composition of `Cel`s that, when played back in sequence, results in an animation.
//!
//! Rather than referring to contained elements (like a `Sprite` inside a `Cel`) by Rust-reference (`&`) or using reference counting,
//! objects are referred to by index. The original object can only be retrieved via a lookup into a collection, which will usually be a
//! global cache of some sort.

use crate::geom_art::{ArtworkSpaceUnit, Point, Size};
use crate::Surface;

#[cfg_attr(
    feature = "serde_support",
    derive(serde::Serialize, serde::Deserialize)
)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Color {
    Opaque(rgb::RGB8),
    Transparent,
}

impl Color {
    /// Creates a new opaque color.
    ///
    /// # Parameters
    /// * `r`: The red component.
    /// * `g`: The green component.
    /// * `b`: The blue component.
    #[inline(always)]
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self::Opaque(rgb::RGB8::new(r, g, b))
    }

    /// Creates a new transparent color.
    #[inline(always)]
    pub fn new_transparent() -> Self {
        Self::Transparent
    }
}

macro_rules! primitive_wrapper {
    ($(#[doc = $doc:expr])* $vis:vis $name:ident < $ty:ty >) => {
        $(#[doc = $doc])*
        #[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize), serde(transparent))]
        #[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
        $vis struct $name($ty);

        impl $name {
            /// Creates a new instance.
            ///
            /// # Arguments
            /// * `value` the value.
            #[inline(always)]
            $vis fn new(value: $ty) -> Self {
                Self(value)
            }

            /// Retrieves the underlying value.
            #[inline(always)]
            $vis fn value(&self) -> $ty {
                self.0
            }

            /// Sets the underlying value.
            #[inline(always)]
            $vis fn set_value(&mut self, value: $ty) {
                self.0 = value;
            }
        }

        impl From<$ty> for $name {
            #[inline(always)]
            fn from(val: $ty) -> Self {
                Self::new(val.into())
            }
        }
    }
}

macro_rules! ref_type {
    ($(#[doc = $doc:expr])* $vis:vis $name:ident < $ty:ty >) => {
        primitive_wrapper!($(#[doc = $doc])* $vis $name<$ty>);

        impl ves_cache::AsIndex for $name {
            fn as_index(&self) -> usize {
                self.0
            }
        }

        impl ves_cache::FromIndex for $name {
            fn from_index(index: usize) -> Self {
                Self::new(index)
            }
        }
    }
}

primitive_wrapper!(
    /// An index into a [`Palette`].
    pub PaletteIndex<u8>
);

ref_type!(
    /// A reference to a [`Palette`].
    pub PaletteRef<usize>
);

/// A palette of colors.
#[cfg_attr(
    feature = "serde_support",
    derive(serde::Serialize, serde::Deserialize)
)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Palette {
    colors: Vec<Color>,
}

impl Palette {
    /// Creates a new instance from a `Vec`.
    pub fn new(colors: Vec<Color>) -> Self {
        Self { colors }
    }

    /// Creates a new instance with the specified length and default value.
    ///
    /// # Parameters
    /// * `length`: The number of entries.
    /// * `default`: The default value.
    pub fn new_filled(length: usize, default: Color) -> Self {
        Self {
            colors: vec![default; length],
        }
    }
}

impl Palette {
    /// Retrieves the number of entries in the palette.
    pub fn len(&self) -> usize {
        self.colors.len()
    }

    /// Determines whether the palette is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Gets an immutable iterator over all slots.
    pub fn iter(&self) -> impl Iterator<Item = (PaletteIndex, &Color)> + '_ {
        self.colors
            .iter()
            .enumerate()
            // Unwrap is OK here because we never add anything other than a PaletteIndex to the Vec
            .map(|(index, color)| (PaletteIndex::new(index.try_into().unwrap()), color))
    }

    /// Gets a mutable iterator over all slots.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (PaletteIndex, &mut Color)> + '_ {
        self.colors
            .iter_mut()
            .enumerate()
            // Unwrap is OK here because we never add anything other than a PaletteIndex to the Vec
            .map(|(index, color)| (PaletteIndex::new(index.try_into().unwrap()), color))
    }
}

impl std::ops::Index<PaletteIndex> for Palette {
    type Output = Color;

    fn index(&self, index: PaletteIndex) -> &Self::Output {
        &self.colors[usize::from(index.value())]
    }
}

impl std::ops::IndexMut<PaletteIndex> for Palette {
    fn index_mut(&mut self, index: PaletteIndex) -> &mut Self::Output {
        &mut self.colors[usize::from(index.value())]
    }
}

#[cfg_attr(
    feature = "serde_support",
    derive(serde::Serialize, serde::Deserialize)
)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct TileSurface {
    data: Vec<PaletteIndex>,
    size: Size,
}

impl TileSurface {
    pub fn new(size: Size) -> Self {
        let data_len = size.width * size.height;
        Self {
            data: vec![PaletteIndex::new(0); data_len.into()],
            size,
        }
    }
}

impl Surface<ArtworkSpaceUnit> for TileSurface {
    type DataType = PaletteIndex;

    fn size(&self) -> Size {
        self.size
    }

    fn data(&self) -> &[Self::DataType] {
        self.data.as_slice()
    }

    fn data_mut(&mut self) -> &mut [Self::DataType] {
        self.data.as_mut_slice()
    }
}

/// A tile. This is the smallest graphical element.
#[cfg_attr(
    feature = "serde_support",
    derive(serde::Serialize, serde::Deserialize)
)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Tile {
    /// The surface.
    surface: TileSurface,
}

impl Tile {
    /// Creates a new instance.
    pub fn new(surface: TileSurface) -> Self {
        Self { surface }
    }

    pub fn surface(&self) -> &TileSurface {
        &self.surface
    }

    pub fn surface_mut(&mut self) -> &mut TileSurface {
        &mut self.surface
    }
}

ref_type!(
    /// A reference to a [`Tile`].
    pub TileRef<usize>
);

/// A sprite. This is basically a [`Tile`] inside a container (like a [`Cel`]) with some extra properties like position and flipping flags.
#[cfg_attr(
    feature = "serde_support",
    derive(serde::Serialize, serde::Deserialize)
)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Sprite {
    /// The tile.
    tile: TileRef,
    /// The palette.
    palette: PaletteRef,
    /// The position of the origin of the tile inside its container.
    position: Point,
    /// A flag that specifies whether the tile is flipped horizontally.
    h_flip: bool,
    /// A flag that specifies whether the tile is flipped vertically.
    v_flip: bool,
}

impl Sprite {
    pub fn new(
        tile: TileRef,
        palette: PaletteRef,
        position: Point,
        h_flip: bool,
        v_flip: bool,
    ) -> Self {
        Self {
            tile,
            palette,
            position,
            h_flip,
            v_flip,
        }
    }

    /// Retrieves the [`TileRef`].
    pub fn tile(&self) -> TileRef {
        self.tile
    }

    /// Retrieves the [`PaletteRef`].
    pub fn palette(&self) -> PaletteRef {
        self.palette
    }

    /// Retrieves the position.
    pub fn position(&self) -> Point {
        self.position
    }

    /// Retrieves the horizontal-flip flag.
    pub fn h_flip(&self) -> bool {
        self.h_flip
    }

    /// Retrieves the vertical-flip flag.
    pub fn v_flip(&self) -> bool {
        self.v_flip
    }
}

/// A cel. This is a composition of zero or more [`Sprite`]s that together form one image.
#[cfg_attr(
    feature = "serde_support",
    derive(serde::Serialize, serde::Deserialize)
)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Cel {
    /// The sprites.
    sprites: Vec<Sprite>,
}

ref_type!(
    /// A reference to a [`Cel`].
    pub CelRef<usize>
);

/// A single frame in an [`Animation`].
#[cfg_attr(
    feature = "serde_support",
    derive(serde::Serialize, serde::Deserialize)
)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct AnimationFrame {
    cel: CelRef,
}

/// An animation. This is a sequence of [`AnimationFrame`]s.
#[cfg_attr(
    feature = "serde_support",
    derive(serde::Serialize, serde::Deserialize)
)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Animation {
    frames: Vec<AnimationFrame>,
}

/// Alternative to `std::panic::catch_unwind()` that is silent in its output.
#[cfg(test)]
fn catch_unwind_silent<F: FnOnce() -> R + std::panic::UnwindSafe, R>(
    f: F,
) -> std::thread::Result<R> {
    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let result = std::panic::catch_unwind(f);
    std::panic::set_hook(prev_hook);
    result
}

#[cfg(test)]
mod test_palette_index {
    use super::PaletteIndex;

    #[test]
    fn test_getters() {
        // Some number
        let idx = PaletteIndex::new(12);
        assert_eq!(idx.value(), 12u8);

        // Zero
        let idx = PaletteIndex::new(0);
        assert_eq!(idx.value(), 0u8);
    }

    #[test]
    fn test_from() {
        // Some number (u8)
        let idx = PaletteIndex::from(12u8);
        assert_eq!(idx.value(), 12u8);

        // Zero (u8)
        let idx = PaletteIndex::from(0u8);
        assert_eq!(idx.value(), 0u8);
    }
}

#[cfg(test)]
mod test_palette {
    use super::{Color, Palette};

    macro_rules! assert_eq_colors {
        ($pal:ident, $($col:expr),*) => {
            {
                let collected = $pal.iter()
                                    .map(|(_,option)| *option)
                                    .collect::<Vec<_>>();
                assert_eq!(collected, [$($col),*]);
            }
        }
    }

    #[test]
    fn test_basics() {
        let color_default = Color::new(255, 0, 255);
        let mut pal = Palette::new_filled(4, color_default);

        assert_eq!(pal.len(), 4);
        assert_eq_colors!(
            pal,
            color_default,
            color_default,
            color_default,
            color_default
        );

        let color0 = Color::new(0xAB, 0xCD, 0xEF);
        let color1 = Color::new(0xAB, 0xCD, 0xEF);
        let color2 = Color::new(0x44, 0x55, 0x66);
        let color3 = Color::new(0x11, 0x22, 0x33);

        pal[2u8.into()] = color2;
        assert_eq_colors!(pal, color_default, color_default, color2, color_default);
        pal[0u8.into()] = color0;
        pal[1u8.into()] = color1;
        pal[3u8.into()] = color3;
        assert_eq_colors!(pal, color0, color1, color2, color3);

        assert_eq!(pal[0u8.into()], color0);
        assert_eq!(pal[1u8.into()], color1);
        assert_eq!(pal[2u8.into()], color2);
        assert_eq!(pal[3u8.into()], color3);

        let result = super::catch_unwind_silent(move || pal[4u8.into()]);
        assert!(result.is_err());
    }
}
