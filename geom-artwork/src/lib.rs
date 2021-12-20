use std::convert::TryFrom;

/// See [`ves_geom::SpaceUnit`].
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
