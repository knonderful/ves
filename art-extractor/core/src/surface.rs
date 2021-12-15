//! A module for working with 2-dimensional surfaces.

use std::ops::{Range, RangeInclusive};
use crate::geom::{ArtworkSpaceUnit, Point, Rect, Size};

/// Local trait for extending `ArtworkSpaceUnit` with `into_usize()`.
pub(crate) trait IntoUsize {
    fn into_usize(self) -> usize;
}

impl IntoUsize for ArtworkSpaceUnit {
    fn into_usize(self) -> usize {
        self.try_into().unwrap()
    }
}

/// A 2-dimensional surface.
pub trait Surface {
    type DataType;

    /// The size.
    fn size(&self) -> Size;

    /// Retrieves a view of the surface.
    fn view(&self, area: Rect) -> SurfaceView;

    /// Retrieves a slice of the raw data.
    fn data(&self) -> &[Self::DataType];

    /// Retrieves a mutable slice of the raw data.
    fn data_mut(&mut self) -> &mut [Self::DataType];

    /// Retrieves the index into the data for the provided position.
    ///
    /// # Parameters
    /// * `position`: The position.
    ///
    /// # Returns
    /// The index or `None` if the provided position is outside of the [`Surface`].
    fn index(&self, position: Point) -> Option<usize>;

    /// Retrieves row data.
    fn row_data(&self, row: &SurfaceRow) -> &[Self::DataType] {
        let indices = row.indices();
        &self.data()[indices.start..indices.end]
    }

    /// Retrieves mutable row data.
    fn row_data_mut(&mut self, row: &SurfaceRow) -> &mut [Self::DataType] {
        let indices = row.indices();
        &mut self.data_mut()[indices.start..indices.end]
    }

    /// Retrieves a reference to the pixel at the provided position.
    ///
    /// # Parameters
    /// * `position`: The position.
    ///
    /// # Returns
    /// The reference or `None` if the provided position is outside of the [`Surface`].
    #[inline(always)]
    fn pixel(&self, position: Point) -> Option<&Self::DataType> {
        self.index(position)
            .map(|index| &self.data()[index])
    }

    /// Retrieves a mutable reference to the pixel at the provided position.
    ///
    /// # Parameters
    /// * `position`: The position.
    ///
    /// # Returns
    /// The reference or `None` if the provided position is outside of the [`Surface`].
    #[inline(always)]
    fn pixel_mut(&mut self, position: Point) -> Option<&mut Self::DataType> {
        self.index(position)
            .map(|index| &mut self.data_mut()[index])
    }
}

/// A row inside a [`Surface`].
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SurfaceRow {
    indices: Range<usize>,
}

impl SurfaceRow {
    fn new(indices: Range<usize>) -> Self {
        Self { indices }
    }

    pub fn indices(&self) -> &Range<usize> {
        &self.indices
    }
}

/// A view into an [`Surface`].
///
/// A view does not contain a reference to the [`Surface`] from which it originates. This is a conscious design choice to keep lifetime and
/// implementation complexity to a minimum. The result is that it is up to the user to apply the [`SurfaceView`] to the correct [`Surface`]
/// when working with the underlying data. For instance, the following code is valid in the eyes of the compiler, although it is _logically_
/// incorrect.
///
/// ```
/// use art_extractor_core::surface::Surface;
/// use art_extractor_core::geom::{Point, Rect, Size};
///
/// fn get_data_from_surf1<'a, 'b>(
///         surf1: &'a impl Surface<DataType=u8>,
///         surf2: &'b impl Surface<DataType=u8>) -> &'b [u8] {
///     let view = surf1.view(Rect::new(Point::new(16, 32), Size::new(16, 16)));
///     surf2.row_data(&view.row_iter().next().unwrap())
/// }
/// ```
///
/// A view never exceeds the area of the original surface.
pub struct SurfaceView {
    surface_width: ArtworkSpaceUnit,
    area: Rect,
}

impl SurfaceView {
    /// Creates a new [`SurfaceView`].
    ///
    /// # Parameters
    /// * `surface`: The surface.
    /// * `area`: The area inside the surface for which to create a view.
    ///
    /// # Panics
    /// This function panics if `area` exceeds the surface.
    pub(crate) fn new(surface: &impl Surface, area: Rect) -> Self {
        let size = surface.size();
        if area.max_x() >= size.width || area.max_y() >= size.height {
            panic!("Area {:?} exceeds surface with dimensions {:?}.", area, size);
        }
        Self {
            surface_width: size.width,
            area,
        }
    }

    /// Creates a [`SurfaceRowIter`].
    pub fn row_iter(&self) -> SurfaceRowIter {
        SurfaceRowIter::new(self.surface_width, &self.area)
    }
}

/// An iterator for [`SurfaceRow`]s.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SurfaceRowIter {
    /// The width of the original surface. This is a `usize` instead of a `ArtworkSpaceUnit` because that's what we're calculating with.
    surface_width: usize,
    /// The width of an output row (normally the width of the view).
    row_width: usize,
    /// The start offset in the surface data.
    start_offset: usize,
    /// The end offset in the surface data.
    end_offset: usize,
    /// The current offset.
    offset: usize,
}

impl SurfaceRowIter {
    fn new(surface_width: ArtworkSpaceUnit, area: &Rect) -> Self {
        let surface_width = surface_width.into_usize();
        let start_offset = surface_width * area.min_y().into_usize() + area.origin.x.into_usize();
        let end_offset = start_offset + surface_width * (area.height().into_usize() - 1);

        Self {
            surface_width,
            row_width: area.width().into_usize(),
            start_offset,
            end_offset,
            offset: start_offset,
        }
    }
}

impl Iterator for SurfaceRowIter {
    type Item = SurfaceRow;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset > self.end_offset {
            return None;
        }

        let range = self.offset..self.offset + self.row_width;
        self.offset += self.surface_width;
        Some(SurfaceRow::new(range))
    }
}

#[cfg(test)]
mod test_surface_row_iter {
    use crate::geom::ArtworkSpaceUnit;
    use super::{SurfaceRow, SurfaceRowIter, IntoUsize};

    #[test]
    fn iteration() {
        const SURFACE_WIDTH: ArtworkSpaceUnit = 256;
        const AREA_X: ArtworkSpaceUnit = 24;
        const AREA_Y: ArtworkSpaceUnit = 32;
        const AREA_WIDTH: ArtworkSpaceUnit = 16;
        const AREA_HEIGHT: ArtworkSpaceUnit = 8;
        let mut iter = SurfaceRowIter::new(SURFACE_WIDTH,
                                           &((AREA_X, AREA_Y), AREA_WIDTH, AREA_HEIGHT).into());
        let mut offset = SURFACE_WIDTH.into_usize() * AREA_Y.into_usize() + AREA_X.into_usize();
        for _ in 0..AREA_HEIGHT {
            assert_eq!(Some(SurfaceRow::new(offset..offset + AREA_WIDTH.into_usize())), iter.next());
            offset += SURFACE_WIDTH.into_usize();
        }
        assert_eq!(None, iter.next());
        assert_eq!(None, iter.next());
    }
}

/// An [`Iterator`] for index offsets of a [`Surface`] axis (x or y).
pub trait SurfaceAxisIterFactory {
    type IterType: Iterator<Item=usize>;

    /// Creates a new [`Iterator`].
    ///
    /// # Parameters
    /// * `min`: The minimal value (inclusive).
    /// * `min`: The maximal value (inclusive).
    fn new_iter(min: usize, max: usize) -> Self::IterType;
}

/// A type that reflects ascending order.
pub struct Ascending;

/// A type that reflects descending order.
pub struct Descending;

impl SurfaceAxisIterFactory for Ascending {
    type IterType = RangeInclusive<usize>;

    fn new_iter(min: usize, max: usize) -> Self::IterType {
        min..=max
    }
}

impl SurfaceAxisIterFactory for Descending {
    type IterType = std::iter::Rev<RangeInclusive<usize>>;

    fn new_iter(min: usize, max: usize) -> Self::IterType {
        Ascending::new_iter(min, max).rev()
    }
}

/// A convenience macro for creating a [`SurfaceIter`].
#[macro_export]
macro_rules! surface_iter {
    ($size:expr, $rect:expr, @hflip, @vflip) => {
        $crate::surface::SurfaceIter::<$crate::surface::Descending, $crate::surface::Descending>::new($size, $rect)
    };
    ($size:expr, $rect:expr, @hflip) => {
        $crate::surface::SurfaceIter::<$crate::surface::Descending, $crate::surface::Ascending>::new($size, $rect)
    };
    ($size:expr, $rect:expr, @vflip) => {
        $crate::surface::SurfaceIter::<$crate::surface::Ascending, $crate::surface::Descending>::new($size, $rect)
    };
    ($size:expr, $rect:expr) => {
        $crate::surface::SurfaceIter::<$crate::surface::Ascending, $crate::surface::Ascending>::new($size, $rect)
    };
}

pub struct SurfaceIter<X, Y> where
    X: SurfaceAxisIterFactory,
    Y: SurfaceAxisIterFactory,
{
    width: usize,
    height: usize,
    x_min: usize,
    x_max: usize,
    x_iter: X::IterType,
    y_iter: Y::IterType,
    row_offset: usize,
}

impl<X, Y> SurfaceIter<X, Y> where
    X: SurfaceAxisIterFactory,
    Y: SurfaceAxisIterFactory,
{
    pub fn new(size_surf: Size, rect_view: Rect) -> Self {
        let width = size_surf.width.into_usize();
        let height = size_surf.height.into_usize();
        let x_min = rect_view.min_x().into_usize();
        let x_max = rect_view.max_x().into_usize();
        let x_iter = X::new_iter(x_min, x_max);
        let mut y_iter = Y::new_iter(rect_view.min_y().into_usize(), rect_view.max_y().into_usize());
        let row_offset = y_iter.next().expect("Expected at least one item in Y-iterator.") * width;
        Self { width, height, x_min, x_max, x_iter, y_iter, row_offset }
    }

    #[inline(always)]
    fn do_next(&mut self) -> Option<usize> {
        match self.x_iter.next() {
            Some(x) => Some(self.row_offset + (x % self.width)),
            None => {
                match self.y_iter.next() {
                    None => None,
                    Some(y) => {
                        self.row_offset = (y % self.height) * self.width;
                        self.x_iter = X::new_iter(self.x_min, self.x_max);
                        self.do_next()
                    }
                }
            }
        }
    }
}

impl<X, Y> Iterator for SurfaceIter<X, Y> where
    X: SurfaceAxisIterFactory,
    Y: SurfaceAxisIterFactory,
{
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        self.do_next()
    }
}

#[cfg(test)]
mod test_surface_iter {
    use crate::geom::{Rect, Size};
    use crate::sprite::GenericSurface;
    use crate::surface::Surface;

    type Surfy = GenericSurface<u8>;

    macro_rules! data {
        ($($elt:expr)*) => {
            [ $($elt,)* ]
        }
    }

    const SOURCE_DATA: [u8; 12 * 8] = data![
        0 0 0 1 1 1 1 1 0 0 0 0
        0 0 1 1 1 1 1 1 1 1 1 0
        0 0 2 2 2 3 3 2 3 0 0 0
        0 2 3 2 3 3 3 2 3 3 3 0
        0 2 3 2 2 3 3 3 2 3 3 3
        0 2 2 3 3 3 3 2 2 2 2 0
        0 0 0 3 3 3 3 3 3 3 0 0
        0 0 1 1 4 1 1 1 1 0 0 0
    ];
    const EMPTY_DATA: [u8; 12 * 8] = data![
        0 0 0 0 0 0 0 0 0 0 0 0
        0 0 0 0 0 0 0 0 0 0 0 0
        0 0 0 0 0 0 0 0 0 0 0 0
        0 0 0 0 0 0 0 0 0 0 0 0
        0 0 0 0 0 0 0 0 0 0 0 0
        0 0 0 0 0 0 0 0 0 0 0 0
        0 0 0 0 0 0 0 0 0 0 0 0
        0 0 0 0 0 0 0 0 0 0 0 0
    ];

    fn copy_data(src_surf: &Surfy, dest_surf: &mut Surfy, src_iter: impl Iterator<Item=usize>, dest_iter: impl Iterator<Item=usize>) {
        let src = src_surf.data();

        let dest = dest_surf.data_mut();
        for (src_idx, dest_idx) in src_iter.zip(dest_iter) {
            //println!("src: {} dest: {}", src_idx, dest_idx);
            dest[dest_idx] = src[src_idx];
        }
    }

    fn create_source() -> Surfy {
        let mut src = Surfy::new(Size::new(12, 8), 0);
        assert_eq!(&EMPTY_DATA, src.data());

        src.data_mut().copy_from_slice(&SOURCE_DATA);
        assert_eq!(&SOURCE_DATA, src.data());
        src
    }

    /// Test with a copy of the entire source surface.
    #[test]
    fn test_full_copy_no_flip() {
        // No flipping
        {
            let src = create_source();
            let mut dest = Surfy::new(src.size(), 0);
            let src_iter = surface_iter!(src.size(), Rect::new((0, 0).into(), src.size()));
            let dest_iter = surface_iter!(dest.size(), Rect::new((0, 0).into(), dest.size()));
            copy_data(&src, &mut dest, src_iter, dest_iter);
            assert_eq!(&SOURCE_DATA, dest.data());
        }

        // H-flip on both
        {
            let src = create_source();
            let mut dest = Surfy::new(src.size(), 0);
            let src_iter = surface_iter!(src.size(), Rect::new((0, 0).into(), src.size()), @hflip);
            let dest_iter = surface_iter!(dest.size(), Rect::new((0, 0).into(), dest.size()), @hflip);
            copy_data(&src, &mut dest, src_iter, dest_iter);
            assert_eq!(&SOURCE_DATA, dest.data());
        }

        // V-flip on both
        {
            let src = create_source();
            let mut dest = Surfy::new(src.size(), 0);
            let src_iter = surface_iter!(src.size(), Rect::new((0, 0).into(), src.size()), @vflip);
            let dest_iter = surface_iter!(dest.size(), Rect::new((0, 0).into(), dest.size()), @vflip);
            copy_data(&src, &mut dest, src_iter, dest_iter);
            assert_eq!(&SOURCE_DATA, dest.data());
        }

        // H-flip and v-flip on both
        {
            let src = create_source();
            let mut dest = Surfy::new(src.size(), 0);
            let src_iter = surface_iter!(src.size(), Rect::new((0, 0).into(), src.size()), @hflip, @vflip);
            let dest_iter = surface_iter!(dest.size(), Rect::new((0, 0).into(), dest.size()), @hflip, @vflip);
            copy_data(&src, &mut dest, src_iter, dest_iter);
            assert_eq!(&SOURCE_DATA, dest.data());
        }
    }

    /// Test with a copy of the entire source surface using horizontal flipping.
    #[test]
    fn test_full_copy_hflip() {
        const EXPECTED: [u8; 12 * 8] = data![
            0 0 0 0 1 1 1 1 1 0 0 0
            0 1 1 1 1 1 1 1 1 1 0 0
            0 0 0 3 2 3 3 2 2 2 0 0
            0 3 3 3 2 3 3 3 2 3 2 0
            3 3 3 2 3 3 3 2 2 3 2 0
            0 2 2 2 2 3 3 3 3 2 2 0
            0 0 3 3 3 3 3 3 3 0 0 0
            0 0 0 1 1 1 1 4 1 1 0 0
        ];

        // H-flip on src
        {
            let src = create_source();
            let mut dest = Surfy::new(src.size(), 0);
            let src_iter = surface_iter!(src.size(), Rect::new((0, 0).into(), src.size()), @hflip);
            let dest_iter = surface_iter!(dest.size(), Rect::new((0, 0).into(), dest.size()));
            copy_data(&src, &mut dest, src_iter, dest_iter);
            assert_eq!(&EXPECTED, dest.data());
        }

        // H-flip on dest
        {
            let src = create_source();
            let mut dest = Surfy::new(src.size(), 0);
            let src_iter = surface_iter!(src.size(), Rect::new((0, 0).into(), src.size()));
            let dest_iter = surface_iter!(dest.size(), Rect::new((0, 0).into(), dest.size()), @hflip);
            copy_data(&src, &mut dest, src_iter, dest_iter);
            assert_eq!(&EXPECTED, dest.data());
        }
    }

    /// Test with a copy of the entire source surface using vertical flipping.
    #[test]
    fn test_full_copy_vflip() {
        const EXPECTED: [u8; 12 * 8] = data![
            0 0 1 1 4 1 1 1 1 0 0 0
            0 0 0 3 3 3 3 3 3 3 0 0
            0 2 2 3 3 3 3 2 2 2 2 0
            0 2 3 2 2 3 3 3 2 3 3 3
            0 2 3 2 3 3 3 2 3 3 3 0
            0 0 2 2 2 3 3 2 3 0 0 0
            0 0 1 1 1 1 1 1 1 1 1 0
            0 0 0 1 1 1 1 1 0 0 0 0
        ];

        // V-flip on src
        {
            let src = create_source();
            let mut dest = Surfy::new(src.size(), 0);
            let src_iter = surface_iter!(src.size(), Rect::new((0, 0).into(), src.size()), @vflip);
            let dest_iter = surface_iter!(dest.size(), Rect::new((0, 0).into(), dest.size()));
            copy_data(&src, &mut dest, src_iter, dest_iter);
            assert_eq!(&EXPECTED, dest.data());
        }

        // V-flip on dest
        {
            let src = create_source();
            let mut dest = Surfy::new(src.size(), 0);
            let src_iter = surface_iter!(src.size(), Rect::new((0, 0).into(), src.size()));
            let dest_iter = surface_iter!(dest.size(), Rect::new((0, 0).into(), dest.size()), @vflip);
            copy_data(&src, &mut dest, src_iter, dest_iter);
            assert_eq!(&EXPECTED, dest.data());
        }
    }

    /// Test with a copy of the entire source surface using both horizontal and vertical flipping.
    #[test]
    fn test_full_copy_hflip_vflip() {
        const EXPECTED: [u8; 12 * 8] = data![
            0 0 0 1 1 1 1 4 1 1 0 0
            0 0 3 3 3 3 3 3 3 0 0 0
            0 2 2 2 2 3 3 3 3 2 2 0
            3 3 3 2 3 3 3 2 2 3 2 0
            0 3 3 3 2 3 3 3 2 3 2 0
            0 0 0 3 2 3 3 2 2 2 0 0
            0 1 1 1 1 1 1 1 1 1 0 0
            0 0 0 0 1 1 1 1 1 0 0 0
        ];

        // H-flip and v-flip on src
        {
            let src = create_source();
            let mut dest = Surfy::new(src.size(), 0);
            let src_iter = surface_iter!(src.size(), Rect::new((0, 0).into(), src.size()), @hflip, @vflip);
            let dest_iter = surface_iter!(dest.size(), Rect::new((0, 0).into(), dest.size()));
            copy_data(&src, &mut dest, src_iter, dest_iter);
            assert_eq!(&EXPECTED, dest.data());
        }

        // H-flip and v-flip on dest
        {
            let src = create_source();
            let mut dest = Surfy::new(src.size(), 0);
            let src_iter = surface_iter!(src.size(), Rect::new((0, 0).into(), src.size()));
            let dest_iter = surface_iter!(dest.size(), Rect::new((0, 0).into(), dest.size()), @hflip, @vflip);
            copy_data(&src, &mut dest, src_iter, dest_iter);
            assert_eq!(&EXPECTED, dest.data());
        }

        // H-flip on src and v-flip on dest
        {
            let src = create_source();
            let mut dest = Surfy::new(src.size(), 0);
            let src_iter = surface_iter!(src.size(), Rect::new((0, 0).into(), src.size()), @hflip);
            let dest_iter = surface_iter!(dest.size(), Rect::new((0, 0).into(), dest.size()), @vflip);
            copy_data(&src, &mut dest, src_iter, dest_iter);
            assert_eq!(&EXPECTED, dest.data());
        }

        // H-flip on dest and v-flip on src
        {
            let src = create_source();
            let mut dest = Surfy::new(src.size(), 0);
            let src_iter = surface_iter!(src.size(), Rect::new((0, 0).into(), src.size()), @vflip);
            let dest_iter = surface_iter!(dest.size(), Rect::new((0, 0).into(), dest.size()), @hflip);
            copy_data(&src, &mut dest, src_iter, dest_iter);
            assert_eq!(&EXPECTED, dest.data());
        }
    }

    #[test]
    fn test_partial_no_wrap() {
        // No flipping
        {
            const EXPECTED: [u8; 12 * 8] = data![
                0 0 0 0 0 0 0 0 0 0 0 0
                0 0 0 0 0 0 0 0 0 0 0 0
                0 0 0 0 0 0 0 0 0 0 0 0
                0 0 0 0 0 0 2 3 2 2 0 0
                0 0 0 0 0 0 2 2 3 3 0 0
                0 0 0 0 0 0 0 0 3 3 0 0
                0 0 0 0 0 0 0 1 1 4 0 0
                0 0 0 0 0 0 0 0 0 0 0 0
            ];
            let src = create_source();
            let mut dest = Surfy::new(src.size(), 0);
            let src_iter = surface_iter!(src.size(), ((1, 4), 4, 4).into());
            let dest_iter = surface_iter!(dest.size(), ((6, 3), 4, 4).into());
            copy_data(&src, &mut dest, src_iter, dest_iter);
            assert_eq!(&EXPECTED, dest.data());
        }

        // H-flip
        {
            const EXPECTED: [u8; 12 * 8] = data![
                0 0 0 0 0 0 0 0 0 0 0 0
                0 0 0 0 0 0 0 0 0 0 0 0
                0 0 0 0 0 0 0 0 0 0 0 0
                0 0 0 0 0 0 2 2 3 2 0 0
                0 0 0 0 0 0 3 3 2 2 0 0
                0 0 0 0 0 0 3 3 0 0 0 0
                0 0 0 0 0 0 4 1 1 0 0 0
                0 0 0 0 0 0 0 0 0 0 0 0
            ];
            let src = create_source();
            let mut dest = Surfy::new(src.size(), 0);
            let src_iter = surface_iter!(src.size(), ((1, 4), 4, 4).into(), @hflip);
            let dest_iter = surface_iter!(dest.size(), ((6, 3), 4, 4).into());
            copy_data(&src, &mut dest, src_iter, dest_iter);
            assert_eq!(&EXPECTED, dest.data());
        }

        // V-flip
        {
            const EXPECTED: [u8; 12 * 8] = data![
                0 0 0 0 0 0 0 0 0 0 0 0
                0 0 0 0 0 0 0 0 0 0 0 0
                0 0 0 0 0 0 0 0 0 0 0 0
                0 0 0 0 0 0 0 1 1 4 0 0
                0 0 0 0 0 0 0 0 3 3 0 0
                0 0 0 0 0 0 2 2 3 3 0 0
                0 0 0 0 0 0 2 3 2 2 0 0
                0 0 0 0 0 0 0 0 0 0 0 0
            ];
            let src = create_source();
            let mut dest = Surfy::new(src.size(), 0);
            let src_iter = surface_iter!(src.size(), ((1, 4), 4, 4).into(), @vflip);
            let dest_iter = surface_iter!(dest.size(), ((6, 3), 4, 4).into());
            copy_data(&src, &mut dest, src_iter, dest_iter);
            assert_eq!(&EXPECTED, dest.data());
        }

        // H-flip and v-flip
        {
            const EXPECTED: [u8; 12 * 8] = data![
                0 0 0 0 0 0 0 0 0 0 0 0
                0 0 0 0 0 0 0 0 0 0 0 0
                0 0 0 0 0 0 0 0 0 0 0 0
                0 0 0 0 0 0 4 1 1 0 0 0
                0 0 0 0 0 0 3 3 0 0 0 0
                0 0 0 0 0 0 3 3 2 2 0 0
                0 0 0 0 0 0 2 2 3 2 0 0
                0 0 0 0 0 0 0 0 0 0 0 0
            ];
            let src = create_source();
            let mut dest = Surfy::new(src.size(), 0);
            let src_iter = surface_iter!(src.size(), ((1, 4), 4, 4).into(), @hflip, @vflip);
            let dest_iter = surface_iter!(dest.size(), ((6, 3), 4, 4).into());
            copy_data(&src, &mut dest, src_iter, dest_iter);
            assert_eq!(&EXPECTED, dest.data());
        }
    }

    /// Test with a copy of a partial surface with horizontal wrapping.
    #[test]
    fn test_partial_h_wrap() {
        // H-wrap on src
        {
            const EXPECTED: [u8; 12 * 8] = data![
                0 0 0 0 0 0 0 0 0 0 0 0
                0 0 0 0 0 0 0 0 0 0 0 0
                0 0 0 0 0 0 0 0 0 0 0 0
                0 0 0 0 0 0 3 3 0 2 0 0
                0 0 0 0 0 0 2 0 0 2 0 0
                0 0 0 0 0 0 0 0 0 0 0 0
                0 0 0 0 0 0 0 0 0 0 0 0
                0 0 0 0 0 0 0 0 0 0 0 0
            ];
            let src = create_source();
            let mut dest = Surfy::new(src.size(), 0);
            let src_iter = surface_iter!(src.size(), ((10, 4), 4, 4).into());
            let dest_iter = surface_iter!(dest.size(), ((6, 3), 4, 4).into());
            copy_data(&src, &mut dest, src_iter, dest_iter);
            assert_eq!(&EXPECTED, dest.data());
        }

        // H-wrap on dest
        {
            const EXPECTED: [u8; 12 * 8] = data![
                0 0 0 0 0 0 0 0 0 0 0 0
                0 0 0 0 0 0 0 0 0 0 0 0
                0 0 0 0 0 0 0 0 0 0 0 0
                2 2 0 0 0 0 0 0 0 0 2 3
                3 3 0 0 0 0 0 0 0 0 2 2
                3 3 0 0 0 0 0 0 0 0 0 0
                1 4 0 0 0 0 0 0 0 0 0 1
                0 0 0 0 0 0 0 0 0 0 0 0
            ];
            let src = create_source();
            let mut dest = Surfy::new(src.size(), 0);
            let src_iter = surface_iter!(src.size(), ((1, 4), 4, 4).into());
            let dest_iter = surface_iter!(dest.size(), ((10, 3), 4, 4).into());
            copy_data(&src, &mut dest, src_iter, dest_iter);
            assert_eq!(&EXPECTED, dest.data());
        }
    }

    /// Test with a copy of a partial surface with vertical wrapping.
    #[test]
    fn test_partial_v_wrap() {
        // V-wrap on src
        {
            const EXPECTED: [u8; 12 * 8] = data![
                0 0 0 0 0 0 0 0 0 0 0 0
                0 0 0 0 0 0 0 0 0 0 0 0
                0 0 0 0 0 0 0 0 0 0 0 0
                0 0 0 0 0 0 0 0 3 3 0 0
                0 0 0 0 0 0 0 1 1 4 0 0
                0 0 0 0 0 0 0 0 1 1 0 0
                0 0 0 0 0 0 0 1 1 1 0 0
                0 0 0 0 0 0 0 0 0 0 0 0
            ];
            let src = create_source();
            let mut dest = Surfy::new(src.size(), 0);
            let src_iter = surface_iter!(src.size(), ((1, 6), 4, 4).into());
            let dest_iter = surface_iter!(dest.size(), ((6, 3), 4, 4).into());
            copy_data(&src, &mut dest, src_iter, dest_iter);
            assert_eq!(&EXPECTED, dest.data());
        }

        // V-wrap on dest
        {
            const EXPECTED: [u8; 12 * 8] = data![
                0 0 0 0 0 0 0 0 3 3 0 0
                0 0 0 0 0 0 0 1 1 4 0 0
                0 0 0 0 0 0 0 0 0 0 0 0
                0 0 0 0 0 0 0 0 0 0 0 0
                0 0 0 0 0 0 0 0 0 0 0 0
                0 0 0 0 0 0 0 0 0 0 0 0
                0 0 0 0 0 0 2 3 2 2 0 0
                0 0 0 0 0 0 2 2 3 3 0 0
            ];
            let src = create_source();
            let mut dest = Surfy::new(src.size(), 0);
            let src_iter = surface_iter!(src.size(), ((1, 4), 4, 4).into());
            let dest_iter = surface_iter!(dest.size(), ((6, 6), 4, 4).into());
            copy_data(&src, &mut dest, src_iter, dest_iter);
            assert_eq!(&EXPECTED, dest.data());
        }
    }
    // TODO: Tests:
    // * Entire surface copy.
    // * View inside surface.
    // * View that extends over X.
    // * View that extends over Y.
    // * View that extends over X and Y.
    // * For all the above: all iteration types.
    // * View that is too big in X, Y and both. (view > surface in some dimension)
    // TIP: Use a simple format like 10x5, that is easy to write expected values in... use simple surface format like u8 or something
}
