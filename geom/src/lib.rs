use std::fmt::Debug;
use std::ops::{Add, Div, Mul, RangeInclusive, Rem, Sub};

/// Returns the value zero (0) for a type.
pub trait Zero {
    /// Returns the value zero.
    fn zero() -> Self;
}

/// Returns the value one (1) for a type.
pub trait One {
    /// Returns the value one.
    fn one() -> Self;
}

/// A unit that is used in a geometrical "space".
///
/// Even though it is possible to simply use some basic data type (e.g. `u32`) as a unit, this is not recommendable in cases for
/// where geometrical "worlds" coexist in the same scope (like an application). For instance, imagine a game that has a top-down
/// world map that can be navigated and a side-view for the levels. Objects and calculations between these worlds should never
/// mix, which can happen easily when the unit type is too basic. A `Point<u32, u32>` from the world map might be used inside a
/// level. Additionally, in case of a game there is another geometrical space: the output surface (usually a window or the entire
/// screen). This is another geometrical space, again with its own unit.
///
/// By wrapping the primitive unit type in an explicit type these spaces can be cleanly separated. Any conversion between spaces
/// (e.g. translating a coordinate from the level view to the screen) must be performed explicitly, thus ruling out any
/// accidental bugs. Additionally, from a code-view perspective the more advanced types are more explicit, making code easier to
/// understand and reason about.
pub trait SpaceUnit:
Copy + Add<Output=Self> + Sub<Output=Self> + Mul<Output=Self> + Div<Output=Self> + Rem<Output=Self> +
Zero + One + From<Self::RawValue> + Ord + PartialOrd
{
    type RawValue;

    fn raw(&self) -> Self::RawValue;
}

/// A point in 2D space.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Point<T> {
    /// The X-coordinate.
    pub x: T,
    /// The Y-coordinate.
    pub y: T,
}

impl<T> Point<T> where
    T: SpaceUnit,
{
    /// Creates a new instance.
    ///
    /// # Parameters
    /// * `x`: The X-coordinate.
    /// * `y`: The X-coordinate.
    #[inline(always)]
    pub fn new(x: T, y: T) -> Self {
        Self {
            x,
            y,
        }
    }

    /// Creates a new instance.
    ///
    /// # Parameters
    /// * `x`: The X-coordinate.
    /// * `y`: The X-coordinate.
    #[inline(always)]
    pub fn new_raw(x: T::RawValue, y: T::RawValue) -> Self {
        Self::new(x.into(), y.into())
    }
}

impl<T> From<(T::RawValue, T::RawValue)> for Point<T> where
    T: SpaceUnit,
{
    #[inline(always)]
    fn from(coords: (T::RawValue, T::RawValue)) -> Self {
        Self::new_raw(coords.0, coords.1)
    }
}

/// A size (or dimension) in 2D space.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Size<T> {
    /// The width.
    pub width: T,
    /// The height.
    pub height: T,
}

impl<T> Size<T> where
    T: SpaceUnit,
{
    /// Creates a new instance.
    ///
    /// # Parameters
    /// * `width`: The width.
    /// * `height`: The height.
    #[inline(always)]
    pub fn new(width: T, height: T) -> Self {
        Self {
            width,
            height,
        }
    }
    /// Creates a new instance.
    ///
    /// # Parameters
    /// * `width`: The width.
    /// * `height`: The height.
    #[inline(always)]
    pub fn new_raw(width: T::RawValue, height: T::RawValue) -> Self {
        Self::new(width.into(), height.into())
    }

    /// Creates a new instance of a square.
    ///
    /// # Parameters
    /// * `side`: The length of a side in pixels.
    #[inline(always)]
    pub fn new_square(side: T) -> Self {
        Self::new(side, side)
    }
}

impl<T> Size<T> where
    T: SpaceUnit,
{
    /// Creates a new instance.
    ///
    /// # Parameters
    /// * `width`: The width.
    /// * `height`: The height.
    #[inline(always)]
    pub fn as_rect(&self) -> Rect<T> {
        Rect::new(Point::new(T::zero(), T::zero()), *self)
    }
}

/// A rectangle in 2D space.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Rect<T> {
    /// The point of origin.
    pub origin: Point<T>,
    /// The size.
    pub size: Size<T>,
}

impl<T> Rect<T> where
    T: SpaceUnit,
{
    /// Creates a new instance.
    ///
    /// # Parameters
    /// * `origin`: The point of origin.
    /// * `size`: The size.
    #[inline(always)]
    pub fn new(origin: Point<T>, size: Size<T>) -> Self {
        Self {
            origin,
            size,
        }
    }

    #[inline(always)]
    pub fn min_x(&self) -> T {
        self.origin.x
    }

    #[inline(always)]
    pub fn min_y(&self) -> T {
        self.origin.y
    }

    #[inline(always)]
    pub fn width(&self) -> T {
        self.size.width
    }

    #[inline(always)]
    pub fn height(&self) -> T {
        self.size.height
    }

    #[inline(always)]
    pub fn max_x(&self) -> T {
        let x = self.origin.x + self.size.width;
        x - T::one()
    }

    #[inline(always)]
    pub fn max_y(&self) -> T {
        self.origin.y + self.size.height - T::one()
    }

    #[inline(always)]
    pub fn range_x(&self) -> RangeInclusive<T> {
        self.min_x()..=self.max_x()
    }

    #[inline(always)]
    pub fn range_y(&self) -> RangeInclusive<T> {
        self.min_y()..=self.max_y()
    }
}

impl<T> From<((T::RawValue, T::RawValue), T::RawValue, T::RawValue)> for Rect<T> where
    T: SpaceUnit,
{
    #[inline(always)]
    fn from(args: ((T::RawValue, T::RawValue), T::RawValue, T::RawValue)) -> Self {
        Self {
            origin: args.0.into(),
            size: Size::<T>::new_raw(args.1, args.2),
        }
    }
}

/// Macro for generating simple [`SpaceUnit`] implementations.
///
/// # Parameters
/// * `name`: Output type name.
/// * `raw_type`: The [`SpaceUnit::RawValue`] type.
#[macro_export]
macro_rules! space_unit {
    ($(#[doc = $doc:expr])* $name:ident, $raw_type:ty) => {
        $(#[doc = $doc])*
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
        pub struct $name($raw_type);

        impl std::ops::Add for $name {
            type Output = Self;

            #[inline(always)]
            fn add(self, rhs: Self) -> Self::Output {
                Self(self.0 + rhs.0)
            }
        }

        impl std::ops::Sub for $name {
            type Output = Self;

            #[inline(always)]
            fn sub(self, rhs: Self) -> Self::Output {
                Self(self.0 - rhs.0)
            }
        }

        impl std::ops::Mul for $name {
            type Output = Self;

            #[inline(always)]
            fn mul(self, rhs: Self) -> Self::Output {
                Self(self.0 * rhs.0)
            }
        }

        impl std::ops::Div for $name {
            type Output = Self;

            #[inline(always)]
            fn div(self, rhs: Self) -> Self::Output {
                Self(self.0 / rhs.0)
            }
        }

        impl std::ops::Rem for $name {
            type Output = Self;

            #[inline(always)]
            fn rem(self, rhs: Self) -> Self::Output {
                Self(self.0 % rhs.0)
            }
        }

        impl ves_geom::Zero for $name {
            #[inline(always)]
            fn zero() -> Self {
                Self(0)
            }
        }

        impl ves_geom::One for $name {
            #[inline(always)]
            fn one() -> Self {
                Self(1)
            }
        }

        impl From<$raw_type> for $name {
            #[inline(always)]
            fn from(value: $raw_type) -> Self {
                Self(value)
            }
        }

        impl ves_geom::SpaceUnit for $name {
            type RawValue = $raw_type;

            #[inline(always)]
            fn raw(&self) -> Self::RawValue {
                self.0
            }
        }
    }
}