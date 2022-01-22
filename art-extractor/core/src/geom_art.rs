//! Module containing geometrical types for "artwork space".

ves_geom::space_unit!(
    /// The unit for "artwork space".
    ArtworkSpaceUnit,
    u32);

impl Into<usize> for ArtworkSpaceUnit {
    #[inline(always)]
    fn into(self) -> usize {
        usize::try_from(self.0).unwrap()
    }
}

/// A point in "artwork space".
///
/// See also [`ArtworkSpaceUnit`].
pub type Point = ves_geom::Point<ArtworkSpaceUnit>;

/// A size in "artwork space".
///
/// See also [`ArtworkSpaceUnit`].
pub type Size = ves_geom::Size<ArtworkSpaceUnit>;

/// A 2-dimensional rectangle in "artwork space".
///
/// See also [`ArtworkSpaceUnit`].
pub type Rect = ves_geom::Rect<ArtworkSpaceUnit>;
