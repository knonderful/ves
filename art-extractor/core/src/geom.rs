//! A module for geometric types for working with 2D artwork.

pub mod generic;

/// The space identifier for the "artwork space". This is the space for working with graphical entities like surfaces, sprites, cels and
/// animations.
///
/// The main reason for having space identifiers is to avoid (unintentionally) mixing up incompatible geometric spaces. For instance,
/// positional data that describes a point in a sprite should not (directly) be used for geometric calculations in the "GUI window space" or
/// the "screen space". Note that it is possible to translate between these spaces, but this should be handled explicitly. Having separate
/// space identifiers avoids accidental errors in this area.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ArtworkSpace;

/// The unit for data in "artwork space".
pub type ArtworkSpaceUnit = u32;

impl crate::IntoUsize for ArtworkSpaceUnit {
    fn into_usize(self) -> usize {
        self.try_into().unwrap()
    }
}

impl generic::Zero for ArtworkSpaceUnit {
    fn zero() -> Self {
        0
    }
}

/// A point in "artwork space".
///
/// See also [`ArtworkSpace`].
pub type Point = generic::Point<ArtworkSpaceUnit, ArtworkSpace>;

/// A size in "artwork space".
///
/// See also [`ArtworkSpace`].
pub type Size = generic::Size<ArtworkSpaceUnit, ArtworkSpace>;

/// A 2-dimensional rectangle in "artwork space".
///
/// See also [`ArtworkSpace`].
pub type Rect = generic::Rect<ArtworkSpaceUnit, ArtworkSpace>;
