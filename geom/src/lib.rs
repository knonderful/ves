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
    /// * `y`: The Y-coordinate.
    #[inline(always)]
    pub fn new(x: impl Into<T>, y: impl Into<T>) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
        }
    }
}

impl<A, B, T> From<(A, B)> for Point<T> where
    A: Into<T>,
    B: Into<T>,
    T: SpaceUnit,
{
    #[inline(always)]
    fn from(coords: (A, B)) -> Self {
        Self::new(coords.0, coords.1)
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
    pub fn new(min: impl Into<Point<T>>, max: impl Into<Point<T>>) -> Self {
        let min: Point<T> = min.into();
        let max: Point<T> = max.into();
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
    pub fn new_from_size(origin: impl Into<Point<T>>, size: Size<T>) -> Self {
        let origin: Point<T> = origin.into();
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

    /// Creates an intersection of this rectangle with the axes defined by the provided point.
    ///
    /// The [`Rect`] on which this method is called will be adjusted to be the top-left rectangle after the intersection.
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
    pub fn intersect_point(&mut self, point: impl Into<Point<T>>) -> RectIntersection<T> {
        let Point { x, y } = point.into();
        let x_start = self.min.x;
        let x_end = self.max.x;
        let y_start = self.min.y;
        let y_end = self.max.y;

        if x_start <= x && x < x_end {
            self.max.x = x;
            let remaining_x = x + T::one();
            if y_start <= y && y < y_end {
                self.max.y = y;
                let remaining_y = y + T::one();
                RectIntersection::Both {
                    top_right: Rect::new(Point::new(remaining_x, y_start), Point::new(x_end, y)),
                    bottom_left: Rect::new(Point::new(x_start, remaining_y), Point::new(x, y_end)),
                    bottom_right: Rect::new(Point::new(remaining_x, remaining_y), Point::new(x_end, y_end)),
                }
            } else {
                RectIntersection::Vertical(
                    Rect::new(Point::new(remaining_x, y_start), Point::new(x_end, y_end))
                )
            }
        } else {
            if y_start <= y && y < y_end {
                self.max.y = y;
                let remaining_y = y + T::one();
                RectIntersection::Horizontal(
                    Rect::new(Point::new(x_start, remaining_y), Point::new(x_end, y_end)),
                )
            } else {
                RectIntersection::None
            }
        }
    }
}

impl<A, B, T> From<(A, B)> for Rect<T> where
    A: Into<Point<T>>,
    B: Into<Point<T>>,
    T: SpaceUnit,
{
    #[inline(always)]
    fn from(args: (A, B)) -> Self {
        Self::new(args.0, args.1)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RectIntersection<T> {
    None,
    Vertical(Rect<T>),
    Horizontal(Rect<T>),
    Both {
        top_right: Rect<T>,
        bottom_left: Rect<T>,
        bottom_right: Rect<T>,
    },
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
        let expected_rect: Rect = ((3, 14), (5, 24)).into();
        let expected_intersection = RectIntersection::Both {
            top_right: ((6, 14), (12, 24)).into(),
            bottom_left: ((3, 25), (5, 30)).into(),
            bottom_right: ((6, 25), (12, 30)).into(),
        };

        let mut rect: Rect = ((3, 14), (12, 30)).into();
        let intersection = rect.intersect_point((5, 24));
        assert_eq!(expected_rect, rect);
        assert_eq!(expected_intersection, intersection);
    }

    #[test]
    fn test_intersect_point_outside_before() {
        let expected_rect: Rect = ((3, 14), (12, 30)).into();
        let expected_intersection = RectIntersection::None;

        let mut rect: Rect = ((3, 14), (12, 30)).into();
        let intersection = rect.intersect_point((2, 12));
        assert_eq!(expected_rect, rect);
        assert_eq!(expected_intersection, intersection);
    }

    #[test]
    fn test_intersect_point_outside_after() {
        let expected_rect: Rect = ((3, 14), (12, 30)).into();
        let expected_intersection = RectIntersection::None;

        let mut rect: Rect = ((3, 14), (12, 30)).into();
        let intersection = rect.intersect_point((14, 31));
        assert_eq!(expected_rect, rect);
        assert_eq!(expected_intersection, intersection);
    }

    #[test]
    fn test_intersect_point_vertical() {
        let expected_rect: Rect = ((3, 14), (5, 30)).into();
        let expected_intersection = RectIntersection::Vertical(((6, 14), (12, 30)).into());

        let mut rect: Rect = ((3, 14), (12, 30)).into();
        let intersection = rect.intersect_point((5, 31));
        assert_eq!(expected_rect, rect);
        assert_eq!(expected_intersection, intersection);
    }

    #[test]
    fn test_intersect_point_horizontal() {
        let expected_rect: Rect = ((3, 14), (12, 24)).into();
        let expected_intersection = RectIntersection::Horizontal(((3, 25), (12, 30)).into());

        let mut rect: Rect = ((3, 14), (12, 30)).into();
        let intersection = rect.intersect_point((2, 24));
        assert_eq!(expected_rect, rect);
        assert_eq!(expected_intersection, intersection);
    }

    #[test]
    fn test_intersect_point_top_left() {
        let expected_rect: Rect = ((3, 14), (3, 14)).into();
        let expected_intersection = RectIntersection::Both {
            top_right: ((4, 14), (12, 14)).into(),
            bottom_left: ((3, 15), (3, 30)).into(),
            bottom_right: ((4, 15), (12, 30)).into(),
        };

        let mut rect: Rect = ((3, 14), (12, 30)).into();
        let intersection = rect.intersect_point((3, 14));
        assert_eq!(expected_rect, rect);
        assert_eq!(expected_intersection, intersection);
    }

    #[test]
    fn test_intersect_point_bottom_right() {
        let expected_rect: Rect = ((3, 14), (12, 30)).into();
        let expected_intersection = RectIntersection::None;

        let mut rect: Rect = ((3, 14), (12, 30)).into();
        let intersection = rect.intersect_point((12, 30));
        assert_eq!(expected_rect, rect);
        assert_eq!(expected_intersection, intersection);
    }
}