use std::fmt::Debug;
use std::marker::PhantomData;

/// A point in 2D space.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Point<T, U> {
    /// The X-coordinate.
    pub x: T,
    /// The Y-coordinate.
    pub y: T,
    _phantom: PhantomData<U>,
}

impl<T, U> Point<T, U> {
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
            _phantom: Default::default(),
        }
    }
}

impl<T, U> From<(T, T)> for Point<T, U> {
    #[inline(always)]
    fn from(coords: (T, T)) -> Self {
        Self::new(coords.0, coords.1)
    }
}

/// A size (or dimension) in 2D space.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Size<T, U> {
    /// The width.
    pub width: T,
    /// The height.
    pub height: T,
    _phantom: PhantomData<U>,
}

impl<T, U> Size<T, U> {
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
            _phantom: Default::default(),
        }
    }
}

/// A rectangle in 2D space.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Rect<T, U> {
    /// The point of origin.
    pub origin: Point<T, U>,
    /// The size.
    pub size: Size<T, U>,
    _phantom: PhantomData<U>,
}

impl<T, U> Rect<T, U> {
    /// Creates a new instance.
    ///
    /// # Parameters
    /// * `origin`: The point of origin.
    /// * `size`: The size.
    #[inline(always)]
    pub fn new(origin: Point<T, U>, size: Size<T, U>) -> Self {
        Self {
            origin,
            size,
            _phantom: Default::default(),
        }
    }
}

impl<T, U> Rect<T, U> where
    T: Copy,
{
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
}


impl<T, U> Rect<T, U> where
    T: Copy + std::ops::Add<Output=T> + std::ops::Sub<Output=T> + From<u8>,
{
    #[inline(always)]
    pub fn max_x(&self) -> T {
        self.origin.x + self.size.width - T::from(1u8)
    }

    #[inline(always)]
    pub fn max_y(&self) -> T {
        self.origin.y + self.size.height - T::from(1u8)
    }
}

impl<T, U> From<((T, T), T, T)> for Rect<T, U> {
    #[inline(always)]
    fn from(args: ((T, T), T, T)) -> Self {
        Self {
            origin: args.0.into(),
            size: Size::<T, U>::new(args.1, args.2),
            _phantom: Default::default(),
        }
    }
}
