#![allow(dead_code)]

use std::marker::PhantomData;
use std::ops::{Range, RangeInclusive};
use std::io::IntoInnerError;

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

    pub fn iter(&self) -> Rectangle2DIter {
        Rectangle2DIter {
            origin_x: self.origin.x,
            pos_x: self.origin.x,
            pos_y: self.origin.y,
            end_x: self.origin.x + self.dimensions.width - 1,
            end_y: self.origin.y + self.dimensions.height - 1,
        }
    }
}

pub struct Rectangle2DIter {
    origin_x: Unit2D,
    pos_x: Unit2D,
    pos_y: Unit2D,
    end_x: Unit2D,
    end_y: Unit2D,
}

impl Iterator for Rectangle2DIter {
    type Item = Position2D;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos_y > self.end_y {
            return None;
        }

        let out = Position2D::new(self.pos_x, self.pos_y);

        if self.pos_x < self.end_x {
            self.pos_x += 1;
        } else {
            self.pos_x = self.origin_x;
            self.pos_y += 1;
        }

        Some(out)
    }
}

/// A drawable 2-dimensional surface.
///
/// A surface can be interpreted in two different ways: as a sequence of bytes or as raster of
/// pixels. The raster has its origin in the top-left corner, i.e. the top-left position is
/// `(0, 0)`, the one to the right of that is `(1, 0)` and the one below that is `(1, 1)`.
pub trait Surface {
    /// The type of pixel value for this surface.
    type PixelValue;

    /// The width of the surface in pixels.
    fn width(&self) -> Unit2D;

    /// The width of the surface in pixels.
    fn height(&self) -> Unit2D;
}

/// A [Surface] that provides [BufferBackedPixel] instances.
pub trait BufferBackedSurface: Surface {
    /// Retrieves the [Pixel] at the provided [Position2D].
    fn pixel(&self, position: Position2D) -> BufferBackedPixel<<Self as Surface>::PixelValue>;
}

/// A [Surface] that provides [BufferBackedPixelMut] instances.
pub trait BufferBackedSurfaceMut: Surface {
    /// Retrieves the [PixelMut] at the provided [Position2D].
    fn pixel_mut(&mut self, position: Position2D) -> BufferBackedPixelMut<<Self as Surface>::PixelValue>;
}

/// A pixel, usually applied to a [Surface].
///
/// The value type can be used for transformation operations on the surface and for conversion
/// between different surface formats.
pub trait Pixel {
    /// The value type.
    type Value;

    /// Retrieves the value.
    fn get_value(&self) -> Self::Value;
}

/// A mutable [Pixel].
pub trait PixelMut {
    /// The value type.
    type Value;

    /// Sets the value.
    fn set_value(&mut self, value: &Self::Value);
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

/// A buffer-backed [Pixel].
///
/// Pixel values are always retrieved from the underlying data buffer.
pub struct BufferBackedPixel<'buf, T> {
    buffer: &'buf [u8],
    _phantom: PhantomData<T>,
}

impl<'buf, T> BufferBackedPixel<'buf, T> {
    pub fn new(buffer: &'buf [u8]) -> Self {
        Self { buffer, _phantom: PhantomData }
    }
}

impl<'buf, T: BufferLoad> Pixel for BufferBackedPixel<'buf, T> {
    type Value = T;

    fn get_value(&self) -> Self::Value {
        <T as BufferLoad>::load(self.buffer)
    }
}

/// A buffer-backed [PixelMut].
///
/// Pixel values are always stored to the underlying data buffer.
pub struct BufferBackedPixelMut<'buf, T> {
    buffer: &'buf mut [u8],
    _phantom: PhantomData<T>,
}

impl<'buf, T> BufferBackedPixelMut<'buf, T> {
    pub fn new(buffer: &'buf mut [u8]) -> Self {
        Self { buffer, _phantom: PhantomData }
    }
}

impl<'buf, T: BufferStore> PixelMut for BufferBackedPixelMut<'buf, T> {
    type Value = T;

    fn set_value(&mut self, value: &T) {
        value.store(self.buffer);
    }
}

#[macro_export]
macro_rules! linear_pixel_buffer {
    ($struct_name: ident, $pixel_type: ty, $width: expr, $height: expr) => {
        pub struct $struct_name {
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
            #[allow(dead_code)]
            pub fn data(&self) -> &[u8] {
                &self.buffer
            }

            #[allow(dead_code)]
            pub fn as_surface(&self) -> impl crate::gfx::BufferBackedSurface<PixelValue=$pixel_type> + '_ {
                crate::gfx::SliceBackedSurface::new(&self.buffer, $width, $height)
            }

            #[allow(dead_code)]
            pub fn as_surface_mut(&mut self) -> impl crate::gfx::BufferBackedSurfaceMut<PixelValue=$pixel_type> + '_ {
                crate::gfx::SliceBackedSurfaceMut::new(&mut self.buffer, $width, $height)
            }
        }
    }
}

macro_rules! slice_backed_surface {
    ($struct_name: ident $($mut:tt)? ) => {
        /// A [Surface] that is backed by a slice.
        pub struct $struct_name<'buf, T> {
            buffer: &'buf $($mut)? [u8],
            width: crate::gfx::Unit2D,
            height: crate::gfx::Unit2D,
            _phantom: PhantomData<T>,
        }

        impl<'buf, T: crate::gfx::BufferMapIndex> $struct_name<'buf, T> {
            /// Creates a new instance.
            pub fn new(buffer: &'buf $($mut)? [u8], width: crate::gfx::Unit2D, height: crate::gfx::Unit2D) -> Self {
                Self { buffer, width, height, _phantom: PhantomData }
            }

            #[inline(always)]
            fn index_wrap(&self, position: crate::gfx::Position2D) -> usize {
                let x = position.x % self.width;
                let y = position.y % self.height;
                (y * self.width + x) as usize
            }

            #[inline(always)]
            fn buffer_range(&self, position: crate::gfx::Position2D) -> core::ops::Range<usize> {
                <T as crate::gfx::BufferMapIndex>::map_index(self.index_wrap(position))
            }
        }

        impl<'buf, T> crate::gfx::Surface for $struct_name<'buf, T> {
            type PixelValue = T;

            fn width(&self) -> crate::gfx::Unit2D {
                self.width
            }

            fn height(&self) -> crate::gfx::Unit2D {
                self.height
            }
        }
    }
}

slice_backed_surface!(SliceBackedSurface);

impl<'buf, T: crate::gfx::BufferMapIndex> BufferBackedSurface for SliceBackedSurface<'buf, T> {
    fn pixel(&self, position: Position2D) -> BufferBackedPixel<Self::PixelValue> {
        let range = self.buffer_range(position);
        crate::gfx::BufferBackedPixel::new(&self.buffer[range.start..range.end])
    }
}

slice_backed_surface!(SliceBackedSurfaceMut mut);

impl<'buf, T: crate::gfx::BufferMapIndex> BufferBackedSurface for SliceBackedSurfaceMut<'buf, T> {
    fn pixel(&self, position: Position2D) -> BufferBackedPixel<Self::PixelValue> {
        let range = self.buffer_range(position);
        crate::gfx::BufferBackedPixel::new(&self.buffer[range.start..range.end])
    }
}

impl<'buf, T: crate::gfx::BufferMapIndex> BufferBackedSurfaceMut for SliceBackedSurfaceMut<'buf, T> {
    fn pixel_mut(&mut self, position: Position2D) -> BufferBackedPixelMut<Self::PixelValue> {
        let range = self.buffer_range(position);
        crate::gfx::BufferBackedPixelMut::new(&mut self.buffer[range.start..range.end])
    }
}

/// An iterator for a [Surface].
///
/// This iterator differs from the standard Rust iterator in that it only supports `for_each()` (for
/// underlying lifetime restriction reasons).
///
/// This iterator only provides immutable access to the surface. For the mutable alternative look to
/// [SurfaceIteratorMut].
pub trait SurfaceIterator {
    /// The type of pixel for the [Surface].
    type PixelType;

    /// Retrieves the next pixel.
    fn next(&mut self) -> Option<BufferBackedPixel<'_, Self::PixelType>>;
}


/// An iterator for a [Surface].
///
/// This iterator differs from the standard Rust iterator in that it only supports `for_each()` (for
/// underlying lifetime restriction reasons).
///
/// This iterator provides mutable access to the surface. For the immutable alternative look to
/// [SurfaceIterator].
pub trait SurfaceIteratorMut {
    /// The type of pixel for the [Surface].
    type PixelType;

    /// Retrieves the next pixel.
    fn next(&mut self) -> Option<BufferBackedPixelMut<'_, Self::PixelType>>;
}

/// The default [SurfaceIterator].
pub struct SurfaceIter<'surf, T> {
    surface: &'surf dyn BufferBackedSurface<PixelValue=T>,
    iterator: Rectangle2DIter,
}

impl<'surf, T> SurfaceIter<'surf, T> {
    /// Creates a new [SurfaceIter] that contains the entire surface area.
    pub fn new(surface: &'surf dyn BufferBackedSurface<PixelValue=T>) -> Self {
        let rect = Rectangle2D::new((0, 0).into(), (surface.width(), surface.height()).into());
        Self::new_with_rectangle(surface, rect)
    }

    /// Creates a new [SurfaceIter] that contains only the provided area.
    pub fn new_with_rectangle(surface: &'surf dyn BufferBackedSurface<PixelValue=T>, rectangle: Rectangle2D) -> Self {
        Self {
            surface,
            iterator: rectangle.iter(),
        }
    }
}

impl<'surf, T> SurfaceIterator for SurfaceIter<'surf, T> {
    type PixelType = T;

    fn next(&mut self) -> Option<BufferBackedPixel<'_, Self::PixelType>> {
        self.iterator.next().map(move |pos| self.surface.pixel(pos))
    }
}

/// The default [SurfaceIteratorMut].
pub struct SurfaceIterMut<'surf, T> {
    surface: &'surf mut dyn BufferBackedSurfaceMut<PixelValue=T>,
    iterator: Rectangle2DIter,
}

impl<'surf, T> SurfaceIterMut<'surf, T> {
    /// Creates a new [SurfaceIterMut] that contains the entire surface area.
    pub fn new(surface: &'surf mut dyn BufferBackedSurfaceMut<PixelValue=T>) -> Self {
        let rect = Rectangle2D::new((0, 0).into(), (surface.width(), surface.height()).into());
        Self::new_with_rectangle(surface, rect)
    }

    /// Creates a new [SurfaceIterMut] that contains only the provided area.
    pub fn new_with_rectangle(surface: &'surf mut dyn BufferBackedSurfaceMut<PixelValue=T>, rectangle: Rectangle2D) -> Self {
        Self {
            surface,
            iterator: rectangle.iter(),
        }
    }
}

impl<'surf, T> SurfaceIteratorMut for SurfaceIterMut<'surf, T> {
    type PixelType = T;

    fn next(&mut self) -> Option<BufferBackedPixelMut<'_, Self::PixelType>> {
        self.iterator.next().map(move |pos| self.surface.pixel_mut(pos))
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
