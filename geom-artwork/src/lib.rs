use std::convert::TryFrom;

/// The geometrical unit for the "artwork space". This is the space for working with graphical entities like surfaces, sprites, cels and
/// animations.
///
/// The main reason for having explicit space units is to avoid (unintentionally) mixing up incompatible geometric spaces. For instance,
/// positional data that describes a point in a sprite should not (directly) be used for geometric calculations in the "GUI window space" or
/// the "screen space". Note that it is possible to translate between these spaces, but this should be handled explicitly. Having separate
/// space units avoids accidental errors in this area.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ArtworkSpaceUnit(u32);

impl std::ops::Add for ArtworkSpaceUnit {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl std::ops::Sub for ArtworkSpaceUnit {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl std::ops::Mul for ArtworkSpaceUnit {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl std::ops::Div for ArtworkSpaceUnit {
    type Output = Self;

    #[inline(always)]
    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}

impl std::ops::Rem for ArtworkSpaceUnit {
    type Output = Self;

    #[inline(always)]
    fn rem(self, rhs: Self) -> Self::Output {
        Self(self.0 % rhs.0)
    }
}

impl ves_geom::Zero for ArtworkSpaceUnit {
    #[inline(always)]
    fn zero() -> Self {
        Self(0)
    }
}

impl ves_geom::One for ArtworkSpaceUnit {
    #[inline(always)]
    fn one() -> Self {
        Self(1)
    }
}

impl Into<usize> for ArtworkSpaceUnit {
    #[inline(always)]
    fn into(self) -> usize {
        usize::try_from(self.0).unwrap()
    }
}

impl From<u32> for ArtworkSpaceUnit {
    #[inline(always)]
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl ves_geom::SpaceUnit for ArtworkSpaceUnit {
    type RawValue = u32;

    #[inline(always)]
    fn raw(&self) -> Self::RawValue {
        self.0
    }
}

impl From<u8> for ArtworkSpaceUnit {
    fn from(val: u8) -> Self {
        Self(val.into())
    }
}

impl From<u16> for ArtworkSpaceUnit {
    fn from(val: u16) -> Self {
        Self(val.into())
    }
}

/// A point in "artwork space".
///
/// See also [`ArtworkSpace`].
pub type Point = ves_geom::Point<ArtworkSpaceUnit>;

/// A size in "artwork space".
///
/// See also [`ArtworkSpace`].
pub type Size = ves_geom::Size<ArtworkSpaceUnit>;

/// A 2-dimensional rectangle in "artwork space".
///
/// See also [`ArtworkSpace`].
pub type Rect = ves_geom::Rect<ArtworkSpaceUnit>;
