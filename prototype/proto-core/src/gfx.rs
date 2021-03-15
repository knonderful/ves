#![allow(dead_code)]

use std::marker::PhantomData;
use std::ops::Range;

/// The unit for [Surface] geometry.
pub type Unit2D = u32;

/// A position in a [Surface].
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Position2D {
    /// The X-coordinate.
    pub x: Unit2D,
    /// The Y-coordinate.
    pub y: Unit2D,
}

impl Position2D {
    pub fn new(x: Unit2D, y: Unit2D) -> Self {
        Self { x, y }
    }
}

impl From<(Unit2D, Unit2D)> for Position2D {
    fn from(tuple: (Unit2D, Unit2D)) -> Self {
        Self::new(tuple.0, tuple.1)
    }
}

/// Dimensions of a body in a [Surface].
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Dimensions2D {
    /// The width.
    pub width: Unit2D,
    /// The height.
    pub height: Unit2D,
}

impl Dimensions2D {
    pub fn new(width: Unit2D, height: Unit2D) -> Self {
        Self { width, height }
    }
}

impl From<(Unit2D, Unit2D)> for Dimensions2D {
    fn from(tuple: (Unit2D, Unit2D)) -> Self {
        Self::new(tuple.0, tuple.1)
    }
}

/// A rectangle in a [Surface].
///
/// The _origin_ defines the top-left point of the rectangle.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Rectangle2D {
    pub origin: Position2D,
    pub dimensions: Dimensions2D,
}

impl Rectangle2D {
    /// Creates a new [Rectangle2D].
    pub fn new(origin: Position2D, dimensions: Dimensions2D) -> Self {
        Self { origin, dimensions }
    }

    /// Returns the range of X values.
    pub fn range_x(&self) -> Range<Unit2D> {
        self.origin.x..self.origin.x + self.dimensions.width
    }

    /// Returns the range of Y values.
    pub fn range_y(&self) -> Range<Unit2D> {
        self.origin.y..self.origin.y + self.dimensions.height
    }

    /// Returns a new [Rectangle2D] with the provided position as an origin.
    pub fn moved(&self, origin: Position2D) -> Self {
        Self { origin, dimensions: self.dimensions }
    }

    pub fn end(&self) -> Position2D {
        Position2D::new(
            self.origin.x + self.dimensions.width - 1,
            self.origin.y + self.dimensions.height - 1
        )
    }
}

struct Rectangle2DIter {
    origin_x: Unit2D,
    position: Position2D,
    end: Position2D,
}

impl Rectangle2DIter {
    fn new(rectangle: Rectangle2D) -> Self {
        Self {
            origin_x: rectangle.origin.x,
            position: rectangle.origin,
            end: rectangle.end(),
        }
    }
}

impl Iterator for Rectangle2DIter {
    type Item = Position2D;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position.y > self.end.y {
            return None;
        }

        let out = Position2D::new(self.position.x, self.position.y);

        if self.position.x < self.end.x {
            self.position.x += 1;
        } else {
            self.position.x = self.origin_x;
            self.position.y += 1;
        }

        Some(out)
    }
}

/// A 2-dimensional surface containing drawable graphics.
pub trait Surface {
    /// The [Dimensions2D] of the surface in pixels.
    fn dimensions(&self) -> Dimensions2D;
}

pub trait SurfaceValueGet {
    type ValueType;

    fn get_value(&self, position: Position2D) -> Self::ValueType;
}

pub trait SurfaceValueSet {
    type ValueType;

    fn set_value(&mut self, position: Position2D, value: &Self::ValueType);
}


/// Describes a type as being linearly stored in a data buffer.
pub trait LinearlyStored {
    /// The number of bits per pixel that the implementing data type requires.
    const BITS_PER_PIXEL: usize;
}

/// A function for mapping from a pixel-based index to a sequence-of-bytes index.
/// See [Surface] for more information on these concepts.
pub trait BufferMapIndex {
    /// Maps a pixel-based index to a sequence-of-bytes index.
    ///
    /// The return type is a [Range], since a single pixel might consist of multiple bytes.
    fn map_index(index: usize) -> Range<usize>;
}

impl<T: LinearlyStored> BufferMapIndex for T {
    fn map_index(index: usize) -> Range<usize> {
        // First calculate everything in bits
        let start = index * Self::BITS_PER_PIXEL;
        let end = start + Self::BITS_PER_PIXEL;
        // Then divide by 8 to get to bytes
        let start = start / 8;
        let end = end / 8;
        start..end
    }
}

/// An entity that can be loaded from a data buffer.
pub trait BufferLoad {
    /// Loads an instance from the provided slice.
    fn load(data: &[u8]) -> Self;
}

/// An entity that can be stored into a data buffer.
pub trait BufferStore {
    /// Stores the instance into the provided slice.
    fn store(&self, data: &mut [u8]);
}

#[macro_export]
macro_rules! linear_pixel_buffer {
    ($struct_vis:vis $struct_name: ident, $pixel_type: ty, $width: expr, $height: expr) => {
        $struct_vis struct $struct_name {
            buffer: [u8; (<$pixel_type as crate::gfx::LinearlyStored>::BITS_PER_PIXEL * $width as usize * $height as usize) / 8],
        }

        impl Default for $struct_name {
            fn default() -> Self {
                Self {
                    buffer: [0; (<$pixel_type as crate::gfx::LinearlyStored>::BITS_PER_PIXEL * $width as usize * $height as usize) / 8]
                }
            }
        }

        impl $struct_name {
            #[inline(always)]
            fn dimensions() -> crate::gfx::Dimensions2D {
                crate::gfx::Dimensions2D::new($width, $height)
            }

            #[allow(dead_code)]
            pub fn data(&self) -> &[u8] {
                &self.buffer
            }

            #[allow(dead_code)]
            pub fn as_surface(&self) -> crate::gfx::SliceBackedSurface<$pixel_type> {
                crate::gfx::SliceBackedSurface::new(&self.buffer, Self::dimensions())
            }

            #[allow(dead_code)]
            pub fn as_surface_mut(&mut self) -> crate::gfx::SliceBackedSurfaceMut<$pixel_type> {
                crate::gfx::SliceBackedSurfaceMut::new(&mut self.buffer, Self::dimensions())
            }
        }
    }
}

macro_rules! slice_backed_surface {
    ($struct_name: ident $($mut:tt)? ) => {
        /// A [Surface] that is backed by a slice.
        pub struct $struct_name<'buf, T> {
            buffer: &'buf $($mut)? [u8],
            dimensions: Dimensions2D,
            _phantom: PhantomData<T>,
        }

        impl<'buf, T: BufferMapIndex> $struct_name<'buf, T> {
            /// Creates a new instance.
            pub fn new(buffer: &'buf $($mut)? [u8], dimensions: Dimensions2D) -> Self {
                Self { buffer, dimensions, _phantom: PhantomData }
            }

            #[inline(always)]
            fn index_wrap(&self, position: Position2D) -> usize {
                (position.y * self.dimensions.width + position.x) as usize
            }

            #[inline(always)]
            fn buffer_range(&self, position: Position2D) -> core::ops::Range<usize> {
                <T as BufferMapIndex>::map_index(self.index_wrap(position))
            }
        }

        impl<'buf, T> Surface for $struct_name<'buf, T> {
            fn dimensions(&self) -> Dimensions2D {
                self.dimensions
            }
        }

        impl<'buf, T: BufferLoad + BufferMapIndex> SurfaceValueGet for $struct_name<'buf, T> {
            type ValueType = T;

            fn get_value(&self, position: Position2D) -> Self::ValueType {
                let range = self.buffer_range(position);
                <Self::ValueType as BufferLoad>::load(&self.buffer[range.start..range.end])
            }
        }
    }
}

slice_backed_surface!(SliceBackedSurface);
slice_backed_surface!(SliceBackedSurfaceMut mut);

impl<'buf, T: BufferStore + BufferMapIndex> SurfaceValueSet for SliceBackedSurfaceMut<'buf, T> {
    type ValueType = T;

    fn set_value(&mut self, position: Position2D, value: &Self::ValueType) {
        let range = self.buffer_range(position);
        value.store(&mut self.buffer[range.start..range.end])
    }
}

struct BoundsWrapper {
    dimensions: Dimensions2D,
}

impl BoundsWrapper {
    fn new(dimensions: Dimensions2D) -> Self {
        Self { dimensions }
    }

    fn wrap_x(&self, x: Unit2D) -> Unit2D {
        x % self.dimensions.width
    }

    fn wrap_y(&self, y: Unit2D) -> Unit2D {
        y % self.dimensions.height
    }

    fn wrap_pos(&self, pos: Position2D) -> Position2D {
        Position2D::new(
            self.wrap_x(pos.x),
            self.wrap_y(pos.y)
        )
    }
}

/// An [Iterator] for iterating over all pixels in a 2-dimensional rectangular area.
///
/// This implementation "wraps around" positions when they exceed the bounds of the specified
/// [Dimensions2D]. For instance, if the dimensions are `200x100`, then position `(212, 101)` will
/// actually correspond to `(12, 1)`.
///
/// The iteration order of positions is not specified, but it is guaranteed to be consistent between
/// [RectangleIterator] instances.
pub struct RectangleIterator {
    iter: Rectangle2DIter,
    bound_wrapper: BoundsWrapper,
}

impl RectangleIterator {
    /// Creates a new [RectangleIterator] for the entire area of the provided dimensions.
    pub fn new(dimensions: Dimensions2D) -> Self {
        Self::new_with_rectangle(dimensions, Rectangle2D::new((0, 0).into(), dimensions))
    }

    /// Creates a new [RectangleIterator] for the specified rectangle inside the provided
    /// dimensions.
    pub fn new_with_rectangle(dimensions: Dimensions2D, rectangle: Rectangle2D) -> Self {
        Self {
            iter: Rectangle2DIter::new(rectangle),
            bound_wrapper: BoundsWrapper::new(dimensions),
        }
    }
}

impl Iterator for RectangleIterator {
    type Item = Position2D;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|pos| self.bound_wrapper.wrap_pos(pos))
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Rgba8888 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl LinearlyStored for Rgba8888 {
    const BITS_PER_PIXEL: usize = 4 * 8;
}

impl BufferLoad for Rgba8888 {
    fn load(data: &[u8]) -> Self {
        Rgba8888 {
            r: data[0],
            g: data[1],
            b: data[2],
            a: data[3],
        }
    }
}

impl BufferStore for Rgba8888 {
    fn store(&self, data: &mut [u8]) {
        data[0] = self.r;
        data[1] = self.g;
        data[2] = self.b;
        data[3] = self.a;
    }
}

impl From<(u8, u8, u8, u8)> for Rgba8888 {
    fn from(tuple: (u8, u8, u8, u8)) -> Self {
        Self {
            r: tuple.0,
            g: tuple.1,
            b: tuple.2,
            a: tuple.3,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Rgb888 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl LinearlyStored for Rgb888 {
    const BITS_PER_PIXEL: usize = 3 * 8;
}

impl BufferLoad for Rgb888 {
    fn load(data: &[u8]) -> Self {
        Rgb888 {
            r: data[0],
            g: data[1],
            b: data[2],
        }
    }
}

impl BufferStore for Rgb888 {
    fn store(&self, data: &mut [u8]) {
        data[0] = self.r;
        data[1] = self.g;
        data[2] = self.b;
    }
}

impl From<(u8, u8, u8)> for Rgb888 {
    fn from(tuple: (u8, u8, u8)) -> Self {
        Self {
            r: tuple.0,
            g: tuple.1,
            b: tuple.2,
        }
    }
}
