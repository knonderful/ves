use std::fmt::{Debug, Display};
use std::ops::{Add, Div, Mul, RangeInclusive, Rem, Sub};

/// Returns the value zero (0) for a type.
pub trait Zero {
    /// Returns the value zero.
    fn zero() -> Self;
}

macro_rules! impl_zero {
    ($ty:ty) => {
        impl Zero for $ty {
            fn zero() -> Self {
                0
            }
        }

    }
}

impl_zero!(u8);
impl_zero!(u16);
impl_zero!(u32);
impl_zero!(u64);
impl_zero!(usize);
impl_zero!(i8);
impl_zero!(i16);
impl_zero!(i32);
impl_zero!(i64);
impl_zero!(isize);

/// Returns the value one (1) for a type.
pub trait One {
    /// Returns the value one.
    fn one() -> Self;
}

macro_rules! impl_one {
    ($ty:ty) => {
        impl One for $ty {
            fn one() -> Self {
                1
            }
        }

    }
}

impl_one!(u8);
impl_one!(u16);
impl_one!(u32);
impl_one!(u64);
impl_one!(usize);
impl_one!(i8);
impl_one!(i16);
impl_one!(i32);
impl_one!(i64);
impl_one!(isize);

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
Zero + One + From<Self::RawValue> + Ord + PartialOrd + Debug
{
    type RawValue;

    fn raw(&self) -> Self::RawValue;
}

/// A finite range.
///
/// This serves as an alterative to the [`core::ops::Range`] family of types that can not be used for iteration when the containing type
/// does not implement [`core::iter::Step`] (which is a nightly-only experimental trait).
pub struct FiniteRange<T> {
    /// The start value (inclusive).
    start: T,
    /// The end value (inclusive).
    end: T,
    /// Flag that signals that iteration is exhausted.
    exhausted: bool,
}

impl<T> FiniteRange<T> where
    T: PartialOrd,
{
    /// Creates a new instance.
    ///
    /// # Parameters
    /// * `start`: The start value (inclusive).
    /// * `end`: The end value (inclusive).
    ///
    /// # Panics
    /// This function panics if `start` is greater than `end`.
    pub fn new(start: T, end: T) -> Self {
        if start > end {
            panic!("Invalid range.");
        }
        Self { start, end, exhausted: false }
    }
}

impl<T> From<(T, T)> for FiniteRange<T> where
    T: PartialOrd + Display,
{
    fn from(value: (T, T)) -> Self {
        FiniteRange::new(value.0, value.1)
    }
}

impl<T> Iterator for FiniteRange<T> where
    T: Copy + PartialOrd + PartialEq + One + Add<Output=T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start == self.end {
            if self.exhausted {
                None
            } else {
                self.exhausted = true;
                Some(self.start)
            }
        } else {
            let out = self.start;
            self.start = self.start + T::one();
            Some(out)
        }
    }
}

impl<T> DoubleEndedIterator for FiniteRange<T> where
    T: Copy + PartialOrd + PartialEq + One + Add<Output=T> + Sub<Output=T>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.end == self.start {
            if self.exhausted {
                None
            } else {
                self.exhausted = true;
                Some(self.end)
            }
        } else {
            let out = self.end;
            self.end = self.end - T::one();
            Some(out)
        }
    }
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
        Rect::new_from_size(Point::new(T::zero(), T::zero()), *self)
    }
}

/// A rectangle in 2D space.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Rect<T> {
    /// The start position (inclusive).
    pub min: Point<T>,
    /// The end position (inclusive).
    pub max: Point<T>,
}

impl<T> Rect<T> where
    T: SpaceUnit,
{
    /// Creates a new instance.
    ///
    /// # Parameters
    /// * `min`: The start position (inclusive).
    /// * `max`: The end position (inclusive).
    #[inline(always)]
    pub fn new(min: Point<T>, max: Point<T>) -> Self {
        assert!(min.x <= max.x || min.y <= max.y, "Invalid min and max: {:?} and {:?}.", min, max);
        Self {
            min,
            max,
        }
    }

    /// Creates a new instance.
    ///
    /// # Parameters
    /// * `origin`: The point of origin.
    /// * `size`: The size.
    #[inline(always)]
    pub fn new_from_size(origin: Point<T>, size: Size<T>) -> Self {
        Self::new(
            origin,
            Point::new(origin.x + size.width - T::one(), origin.y + size.height - T::one()),
        )
    }

    #[inline(always)]
    pub fn min_x(&self) -> T {
        self.min.x
    }

    #[inline(always)]
    pub fn min_y(&self) -> T {
        self.min.y
    }

    #[inline(always)]
    pub fn width(&self) -> T {
        (self.max.x - self.min.x) + T::one()
    }

    #[inline(always)]
    pub fn height(&self) -> T {
        (self.max.y - self.min.y) + T::one()
    }

    #[inline(always)]
    pub fn max_x(&self) -> T {
        self.max.x
    }

    #[inline(always)]
    pub fn max_y(&self) -> T {
        self.max.y
    }

    #[inline(always)]
    pub fn range_x(&self) -> RangeInclusive<T> {
        self.min_x()..=self.max_x()
    }

    #[inline(always)]
    pub fn range_y(&self) -> RangeInclusive<T> {
        self.min_y()..=self.max_y()
    }

    #[inline(always)]
    pub fn min(&self) -> Point<T> {
        self.min
    }

    #[inline(always)]
    pub fn max(&self) -> Point<T> {
        self.max
    }

    #[inline(always)]
    pub fn size(&self) -> Size<T> {
        Size::new(self.width(), self.height())
    }
}

impl<T> From<((T::RawValue, T::RawValue), (T::RawValue, T::RawValue))> for Rect<T> where
    T: SpaceUnit,
{
    #[inline(always)]
    fn from(args: ((T::RawValue, T::RawValue), (T::RawValue, T::RawValue))) -> Self {
        Self::new(args.0.into(), args.1.into())
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

        impl $crate::Zero for $name {
            #[inline(always)]
            fn zero() -> Self {
                Self(0)
            }
        }

        impl $crate::One for $name {
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

        impl $crate::SpaceUnit for $name {
            type RawValue = $raw_type;

            #[inline(always)]
            fn raw(&self) -> Self::RawValue {
                self.0
            }
        }
    }
}