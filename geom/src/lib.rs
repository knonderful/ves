//! A library for working with types in a 2-dimensional space.
//!
//! Every type is generic over `T`, where `T` is the so-called "space unit". The type of `T` should be specific to a certain space or
//! domain.
//!
//! Even though it is possible to simply use some basic data type (e.g. `u32`) as a unit, this is not recommendable in cases for
//! where geometrical "worlds" coexist in the same scope (like an application). For instance, imagine a game that has a top-down
//! world map that can be navigated and a side-view for the levels. Objects and calculations between these worlds should never
//! mix, which can happen easily when the unit type is too basic. A `Point<u32, u32>` from the world map might be used inside a
//! level. Additionally, in case of a game there is another geometrical space: the output surface (usually a window or the entire
//! screen). This is another geometrical space, again with its own unit.
//!
//! By wrapping the primitive unit type in an explicit type these spaces can be cleanly separated. Any conversion between spaces
//! (e.g. translating a coordinate from the level view to the screen) must be performed explicitly, thus ruling out any
//! accidental bugs. Additionally, from a code-view perspective the more advanced types are more explicit, making code easier to
//! understand and reason about.

use std::fmt::{Debug, Formatter};
use std::ops::{Add, RangeInclusive, Sub};

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
    };
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
    };
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

impl<T> FiniteRange<T>
where
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
        Self {
            start,
            end,
            exhausted: false,
        }
    }
}

impl<T> From<(T, T)> for FiniteRange<T>
where
    T: PartialOrd,
{
    fn from(value: (T, T)) -> Self {
        FiniteRange::new(value.0, value.1)
    }
}

impl<T> Iterator for FiniteRange<T>
where
    T: Copy + PartialOrd + PartialEq + One + Add<Output = T>,
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

impl<T> DoubleEndedIterator for FiniteRange<T>
where
    T: Copy + PartialOrd + PartialEq + One + Add<Output = T> + Sub<Output = T>,
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
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Point<T> {
    /// The X-coordinate.
    pub x: T,
    /// The Y-coordinate.
    pub y: T,
}

impl<T> Debug for Point<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("({:?}, {:?})", self.x, self.y))
    }
}

impl<T> Point<T> {
    /// Creates a new instance.
    ///
    /// # Parameters
    /// * `x`: The X-coordinate.
    /// * `y`: The Y-coordinate.
    #[inline(always)]
    pub fn new(x: impl Into<T>, y: impl Into<T>) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
        }
    }
}

impl<A, B, T> From<(A, B)> for Point<T>
where
    A: Into<T>,
    B: Into<T>,
{
    #[inline(always)]
    fn from(coords: (A, B)) -> Self {
        Self::new(coords.0, coords.1)
    }
}

/// A size (or dimension) in 2D space.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Size<T> {
    /// The width.
    pub width: T,
    /// The height.
    pub height: T,
}

impl<T> Debug for Size<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}x{:?}", self.width, self.height))
    }
}

impl<T> Size<T> {
    /// Creates a new instance.
    ///
    /// # Parameters
    /// * `width`: The width.
    /// * `height`: The height.
    #[inline(always)]
    pub fn new(width: impl Into<T>, height: impl Into<T>) -> Self {
        Self {
            width: width.into(),
            height: height.into(),
        }
    }
}

impl<T> Size<T>
where
    T: Copy,
{
    /// Creates a new instance of a square.
    ///
    /// # Parameters
    /// * `side`: The length of a side in pixels.
    #[inline(always)]
    pub fn new_square(side: impl Into<T>) -> Self {
        // T is copy, but impl Into<T> isn't, so we need to take the T and then we can get around move problems
        let side = side.into();
        Self::new(side, side)
    }
}

impl<T> Size<T>
where
    T: Copy + Add<Output = T> + Sub<Output = T> + Zero + PartialOrd + Debug + One,
{
    /// Creates a new instance.
    ///
    /// # Parameters
    /// * `width`: The width.
    /// * `height`: The height.
    #[inline(always)]
    pub fn as_rect(&self) -> Rect<T> {
        Rect::new_from_size((T::zero(), T::zero()), *self)
    }
}

/// A rectangle in 2D space.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Rect<T> {
    /// The start position (inclusive).
    pub min: Point<T>,
    /// The end position (inclusive).
    pub max: Point<T>,
}

impl<T> Debug for Rect<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("({:?}, {:?})", self.min, self.max))
    }
}

impl<T> Rect<T>
where
    T: Copy + PartialOrd + PartialEq + Debug,
{
    /// Creates a new instance.
    ///
    /// # Parameters
    /// * `min`: The start position (inclusive).
    /// * `max`: The end position (inclusive).
    #[inline(always)]
    pub fn new(min: impl Into<Point<T>>, max: impl Into<Point<T>>) -> Self {
        let min: Point<T> = min.into();
        let max: Point<T> = max.into();
        assert!(
            min.x <= max.x || min.y <= max.y,
            "Invalid min and max: {:?} and {:?}.",
            min,
            max
        );
        Self { min, max }
    }
}

impl<T> Rect<T>
where
    T: Copy + Add<Output = T> + Sub<Output = T> + PartialOrd + PartialEq + Debug + One,
{
    /// Creates a new instance.
    ///
    /// # Parameters
    /// * `origin`: The point of origin.
    /// * `size`: The size.
    #[inline(always)]
    pub fn new_from_size(origin: impl Into<Point<T>>, size: Size<T>) -> Self {
        let origin: Point<T> = origin.into();
        Self::new(
            origin,
            (
                origin.x + size.width - T::one(),
                origin.y + size.height - T::one(),
            ),
        )
    }
}

impl<T> Rect<T>
where
    T: Copy,
{
    #[inline(always)]
    pub fn min_x(&self) -> T {
        self.min.x
    }

    #[inline(always)]
    pub fn min_y(&self) -> T {
        self.min.y
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
}

impl<T> Rect<T>
where
    T: Copy + Add<Output = T> + Sub<Output = T> + One,
{
    #[inline(always)]
    pub fn width(&self) -> T {
        (self.max.x - self.min.x) + T::one()
    }

    #[inline(always)]
    pub fn height(&self) -> T {
        (self.max.y - self.min.y) + T::one()
    }

    #[inline(always)]
    pub fn size(&self) -> Size<T> {
        Size::new(self.width(), self.height())
    }
}

impl<T> Rect<T>
where
    T: Copy + Add<Output = T> + PartialOrd + PartialEq + Debug + One,
{
    /// Creates an intersection of this rectangle with the axes defined by the provided point.
    ///
    /// # Parameters
    /// - `point`: A [`Point`] that specifies the X- and Y-axis for the intersection. The axes themselves will be part of the top-left rectangle after intersection.
    ///
    /// # Example
    ///
    /// ```example
    ///    3     6     9           3     6     9
    /// 12 +-----------+        12 +-----+ +---+
    ///    |           |           |     | |   |
    ///    |           |           |     | |   |
    ///    |           |  ===>     |     | |   |
    /// 16 |     x     |        16 +-----+ +---+
    ///    |           |        17 +-----+ +---+
    ///    |           |           |     | |   |
    /// 19 +-----------+        19 +-----+ +---+
    /// ```
    pub fn intersect_point(&self, point: impl Into<Point<T>>) -> RectIntersection<T> {
        let Point { x, y } = point.into();
        let x_start = self.min.x;
        let x_end = self.max.x;
        let y_start = self.min.y;
        let y_end = self.max.y;

        if x_start <= x && x < x_end {
            let remaining_x = x + T::one();
            if y_start <= y && y < y_end {
                let remaining_y = y + T::one();
                RectIntersection::Both {
                    top_left: ((x_start, y_start), (x, y)).into(),
                    top_right: ((remaining_x, y_start), (x_end, y)).into(),
                    bottom_left: ((x_start, remaining_y), (x, y_end)).into(),
                    bottom_right: ((remaining_x, remaining_y), (x_end, y_end)).into(),
                }
            } else {
                RectIntersection::Vertical {
                    left: ((x_start, y_start), (x, y_end)).into(),
                    right: ((remaining_x, y_start), (x_end, y_end)).into(),
                }
            }
        } else if y_start <= y && y < y_end {
            let remaining_y = y + T::one();
            RectIntersection::Horizontal {
                top: ((x_start, y_start), (x_end, y)).into(),
                bottom: ((x_start, remaining_y), (x_end, y_end)).into(),
            }
        } else {
            RectIntersection::None
        }
    }
}

impl<A, B, T> From<(A, B)> for Rect<T>
where
    A: Into<Point<T>>,
    B: Into<Point<T>>,
    T: Copy + PartialOrd + PartialEq + Debug,
{
    #[inline(always)]
    fn from(args: (A, B)) -> Self {
        Self::new(args.0, args.1)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RectIntersection<T> {
    None,
    Vertical {
        left: Rect<T>,
        right: Rect<T>,
    },
    Horizontal {
        top: Rect<T>,
        bottom: Rect<T>,
    },
    Both {
        top_left: Rect<T>,
        top_right: Rect<T>,
        bottom_left: Rect<T>,
        bottom_right: Rect<T>,
    },
}

impl<T> RectIntersection<T> {
    pub fn for_each(&self, mut func: impl FnMut(&Rect<T>)) {
        match self {
            RectIntersection::None => {}
            RectIntersection::Vertical { left, right } => {
                func(left);
                func(right);
            }
            RectIntersection::Horizontal { top, bottom } => {
                func(top);
                func(bottom);
            }
            RectIntersection::Both {
                top_left,
                top_right,
                bottom_left,
                bottom_right,
            } => {
                func(top_left);
                func(top_right);
                func(bottom_left);
                func(bottom_right);
            }
        }
    }
}

/// Macro for generating simple "space unit" implementations.
///
/// # Parameters
/// * `name`: Output type name.
/// * `raw_type`: The raw (inner) value type.
#[macro_export]
macro_rules! space_unit {
    ($(#[doc = $doc:expr])* $name:ident, $raw_type:ty) => {
        $(#[doc = $doc])*
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
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

        impl core::fmt::Debug for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                core::fmt::Debug::fmt(&self.0, f)
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

        impl  $name {
            #[inline(always)]
            pub fn raw(&self) -> $raw_type {
                self.0
            }
        }
    }
}

#[cfg(test)]
space_unit!(
    /// A space unit for tests.
    TestSpaceUnit,
    u16
);

#[cfg(test)]
mod test_rect {
    use super::TestSpaceUnit;

    type Rect = super::Rect<TestSpaceUnit>;
    type RectIntersection = super::RectIntersection<TestSpaceUnit>;

    #[test]
    fn test_intersect_point_inside() {
        let expected_intersection = RectIntersection::Both {
            top_left: ((3, 14), (5, 24)).into(),
            top_right: ((6, 14), (12, 24)).into(),
            bottom_left: ((3, 25), (5, 30)).into(),
            bottom_right: ((6, 25), (12, 30)).into(),
        };

        let rect: Rect = ((3, 14), (12, 30)).into();
        let intersection = rect.intersect_point((5, 24));
        assert_eq!(expected_intersection, intersection);
    }

    #[test]
    fn test_intersect_point_outside_before() {
        let expected_intersection = RectIntersection::None;

        let rect: Rect = ((3, 14), (12, 30)).into();
        let intersection = rect.intersect_point((2, 12));
        assert_eq!(expected_intersection, intersection);
    }

    #[test]
    fn test_intersect_point_outside_after() {
        let expected_intersection = RectIntersection::None;

        let rect: Rect = ((3, 14), (12, 30)).into();
        let intersection = rect.intersect_point((14, 31));
        assert_eq!(expected_intersection, intersection);
    }

    #[test]
    fn test_intersect_point_vertical() {
        let expected_intersection = RectIntersection::Vertical {
            left: ((3, 14), (5, 30)).into(),
            right: ((6, 14), (12, 30)).into(),
        };

        let rect: Rect = ((3, 14), (12, 30)).into();
        let intersection = rect.intersect_point((5, 31));
        assert_eq!(expected_intersection, intersection);
    }

    #[test]
    fn test_intersect_point_horizontal() {
        let expected_intersection = RectIntersection::Horizontal {
            top: ((3, 14), (12, 24)).into(),
            bottom: ((3, 25), (12, 30)).into(),
        };

        let rect: Rect = ((3, 14), (12, 30)).into();
        let intersection = rect.intersect_point((2, 24));
        assert_eq!(expected_intersection, intersection);
    }

    #[test]
    fn test_intersect_point_top_left() {
        let expected_intersection = RectIntersection::Both {
            top_left: ((3, 14), (3, 14)).into(),
            top_right: ((4, 14), (12, 14)).into(),
            bottom_left: ((3, 15), (3, 30)).into(),
            bottom_right: ((4, 15), (12, 30)).into(),
        };

        let rect: Rect = ((3, 14), (12, 30)).into();
        let intersection = rect.intersect_point((3, 14));
        assert_eq!(expected_intersection, intersection);
    }

    #[test]
    fn test_intersect_point_bottom_right() {
        let expected_intersection = RectIntersection::None;

        let rect: Rect = ((3, 14), (12, 30)).into();
        let intersection = rect.intersect_point((12, 30));
        assert_eq!(expected_intersection, intersection);
    }
}
