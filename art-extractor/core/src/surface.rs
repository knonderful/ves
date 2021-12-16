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

/// An [`Iterator`] factory for index offsets of a [`Surface`] axis (x or y).
pub trait SurfaceAxisIterFactory {
    type IterType: Iterator<Item=usize>;

    /// Creates a new [`Iterator`].
    ///
    /// # Parameters
    /// * `min`: The minimal value (inclusive).
    /// * `max`: The maximal value (inclusive).
    /// * `limit`: The natural limit for indices on this axis (exclusive). For the X-axis this is normally the surface width and for the Y-axis this is the surface height.
    ///
    /// # Returns
    /// The [`Iterator`] or a [`String`] with a description of the error.
    fn new_iter(min: usize, max: usize, limit: usize) -> Result<Self::IterType, String>;
}

fn check_min_max(min: usize, max: usize) -> Result<(), String> {
    if min == max {
        Err(String::from("Min and max are equal."))
    } else if min > max {
        Err(String::from("Min is greater than max."))
    } else {
        Ok(())
    }
}

/// A [`SurfaceAxisIterFactory`] with ascending iteration order and in which bounds are not checked.
struct AscendingUnchecked;

impl SurfaceAxisIterFactory for AscendingUnchecked {
    type IterType = RangeInclusive<usize>;

    fn new_iter(min: usize, max: usize, _limit: usize) -> Result<Self::IterType, String> {
        check_min_max(min, max)?;
        Ok(min..=max)
    }
}

/// A [`SurfaceAxisIterFactory`] with descending iteration order and in which bounds are not checked.
struct DescendingUnchecked;

impl SurfaceAxisIterFactory for DescendingUnchecked {
    type IterType = std::iter::Rev<RangeInclusive<usize>>;

    fn new_iter(min: usize, max: usize, limit: usize) -> Result<Self::IterType, String> {
        check_min_max(min, max)?;
        AscendingUnchecked::new_iter(min, max, limit).map(Iterator::rev)
    }
}

fn check_limit(max: usize, limit: usize) -> Result<(), String> {
    if max >= limit {
        Err(String::from("Max is out of bounds."))
    } else {
        Ok(())
    }
}

/// A [`SurfaceAxisIterFactory`] with descending iteration order.
pub struct Ascending;

impl SurfaceAxisIterFactory for Ascending {
    type IterType = RangeInclusive<usize>;

    fn new_iter(min: usize, max: usize, limit: usize) -> Result<Self::IterType, String> {
        check_limit(max, limit)?;
        AscendingUnchecked::new_iter(min, max, limit)
    }
}

/// A [`SurfaceAxisIterFactory`] with descending iteration order.
pub struct Descending;

impl SurfaceAxisIterFactory for Descending {
    type IterType = std::iter::Rev<RangeInclusive<usize>>;

    fn new_iter(min: usize, max: usize, limit: usize) -> Result<Self::IterType, String> {
        check_limit(max, limit)?;
        DescendingUnchecked::new_iter(min, max, limit)
    }
}

pub struct Modularizer<I> {
    iter: I,
    limit: usize,
}

impl<I> Modularizer<I> {
    fn new(iter: I, limit: usize) -> Self {
        Self { iter, limit }
    }
}

impl<I> Iterator for Modularizer<I> where
    I: Iterator<Item=usize>,
{
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|val| val % self.limit)
    }
}


/// A [`SurfaceAxisIterFactory`] with ascending iteration order. This implementation will wrap-around on axis boundaries.
pub struct AscendingWrap;

impl SurfaceAxisIterFactory for AscendingWrap {
    type IterType = Modularizer<RangeInclusive<usize>>;

    fn new_iter(min: usize, max: usize, limit: usize) -> Result<Self::IterType, String> {
        AscendingUnchecked::new_iter(min, max, limit)
            .map(|iter| Modularizer::new(iter, limit))
    }
}

/// A [`SurfaceAxisIterFactory`] with descending iteration order. This implementation will wrap-around on axis boundaries.
pub struct DescendingWrap;

impl SurfaceAxisIterFactory for DescendingWrap {
    type IterType = Modularizer<std::iter::Rev<RangeInclusive<usize>>>;

    fn new_iter(min: usize, max: usize, limit: usize) -> Result<Self::IterType, String> {
        DescendingUnchecked::new_iter(min, max, limit)
            .map(|iter| Modularizer::new(iter, limit))
    }
}

pub struct SurfaceIter<X = Ascending, Y = Ascending> where
    X: SurfaceAxisIterFactory,
    Y: SurfaceAxisIterFactory,
{
    width: usize,
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
    pub fn new(size_surf: Size, rect_view: Rect) -> Result<Self, String> {
        let width = size_surf.width.into_usize();
        let height = size_surf.height.into_usize();
        let x_min = rect_view.min_x().into_usize();
        let x_max = rect_view.max_x().into_usize();
        let x_iter = X::new_iter(x_min, x_max, width)
            .map_err(|msg| format!("Could not create iterator for X-axis (min: {}, max: {}, limit: {}): {}", x_min, x_max, width, msg))?;
        let y_min = rect_view.min_y().into_usize();
        let y_max = rect_view.max_y().into_usize();
        let mut y_iter = Y::new_iter(y_min, y_max, height)
            .map_err(|msg| format!("Could not create iterator for Y-axis (min: {}, max: {}, limit: {}): {}", y_min, y_max, height, msg))?;
        let row_offset = y_iter.next().ok_or_else(|| "Expected at least one item in Y-iterator.")? * width;
        Ok(Self { width, x_min, x_max, x_iter, y_iter, row_offset })
    }

    #[inline(always)]
    fn do_next(&mut self) -> Option<usize> {
        match self.x_iter.next() {
            Some(x) => Some(self.row_offset + x),
            None => {
                match self.y_iter.next() {
                    None => None,
                    Some(y) => {
                        self.row_offset = y * self.width;
                        // We're forced to unwrap here, since we can't return an error, but it should also not fail because we called this
                        // with the same params in the constructor.
                        self.x_iter = X::new_iter(self.x_min, self.x_max, self.width).unwrap();
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


    /// A convenience macro for creating a [`SurfaceIter`].
    macro_rules! surface_iter {
        ($size:expr, $rect:expr, @hflip, @vflip) => {
            $crate::surface::SurfaceIter::<$crate::surface::DescendingWrap, $crate::surface::DescendingWrap>::new($size, $rect).unwrap()
        };
        ($size:expr, $rect:expr, @hflip) => {
            $crate::surface::SurfaceIter::<$crate::surface::DescendingWrap, $crate::surface::AscendingWrap>::new($size, $rect).unwrap()
        };
        ($size:expr, $rect:expr, @vflip) => {
            $crate::surface::SurfaceIter::<$crate::surface::AscendingWrap, $crate::surface::DescendingWrap>::new($size, $rect).unwrap()
        };
        ($size:expr, $rect:expr) => {
            $crate::surface::SurfaceIter::<$crate::surface::AscendingWrap, $crate::surface::AscendingWrap>::new($size, $rect).unwrap()
        };
    }

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

    /// Test with a copy of a partial surface with no wrapping.
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


    /// Test with a copy of a partial surface with both horizontal and vertical wrapping.
    #[test]
    fn test_partial_hv_wrap() {
        // H-wrap and v-wrap on src
        {
            const EXPECTED: [u8; 12 * 8] = data![
                0 0 0 0 0 0 0 0 0 0 0 0
                0 0 0 0 0 0 0 0 0 0 0 0
                0 0 0 0 0 0 0 0 0 0 0 0
                0 0 0 0 0 0 0 0 0 0 0 0
                0 0 0 0 0 0 0 0 0 0 0 0
                0 0 0 0 0 0 0 0 0 0 0 0
                0 0 0 0 0 0 1 0 0 0 0 0
                0 0 0 0 0 0 0 0 0 0 0 0
            ];
            let src = create_source();
            let mut dest = Surfy::new(src.size(), 0);
            let src_iter = surface_iter!(src.size(), ((10, 6), 4, 4).into());
            let dest_iter = surface_iter!(dest.size(), ((6, 3), 4, 4).into());
            copy_data(&src, &mut dest, src_iter, dest_iter);
            assert_eq!(&EXPECTED, dest.data());
        }

        // H-wrap and v-wrap on dest
        {
            const EXPECTED: [u8; 12 * 8] = data![
                3 3 0 0 0 0 0 0 0 0 0 0
                1 4 0 0 0 0 0 0 0 0 0 1
                0 0 0 0 0 0 0 0 0 0 0 0
                0 0 0 0 0 0 0 0 0 0 0 0
                0 0 0 0 0 0 0 0 0 0 0 0
                0 0 0 0 0 0 0 0 0 0 0 0
                2 2 0 0 0 0 0 0 0 0 2 3
                3 3 0 0 0 0 0 0 0 0 2 2
            ];
            let src = create_source();
            let mut dest = Surfy::new(src.size(), 0);
            let src_iter = surface_iter!(src.size(), ((1, 4), 4, 4).into());
            let dest_iter = surface_iter!(dest.size(), ((10, 6), 4, 4).into());
            copy_data(&src, &mut dest, src_iter, dest_iter);
            assert_eq!(&EXPECTED, dest.data());
        }
    }
}

/// Iterates over the indices for a selection in a pair of [`Surface`]s and passes the indices to the provided function.
///
/// # Parameters
/// * `a_surf_size`: The size of the first surface.
/// * `a_select_rect`: The selection rectangle in the first surface.
/// * `b_surf_size`: The size of the second surface.
/// * `b_select_origin`: The point of origin of the selection rectangle in the second surface.
/// * `hflip`: A flag indicating that the iteration order on the horizontal axis should be inversed.
/// * `vflip`: A flag indicating that the iteration order on the vertical axis should be inversed.
/// * `func`: The function to call for every index.
///
/// # Returns
/// `Err` of a selection entirely exceeds a surface bound, otherwise `Ok`.
pub fn surface_iterate_2<F>(a_surf_size: Size, a_select_rect: Rect, b_surf_size: Size, b_select_origin: Point, hflip: bool, vflip: bool, mut func: F) -> Result<(), String> where
    F: FnMut(usize, usize)
{
    let b_select_rect = Rect::new(b_select_origin, a_select_rect.size);
    let src_x_wrap = a_select_rect.max_x() >= a_surf_size.width;
    let src_y_wrap = a_select_rect.max_y() >= a_surf_size.height;
    let dest_x_wrap = b_select_rect.max_x() >= b_surf_size.width;
    let dest_y_wrap = b_select_rect.max_y() >= b_surf_size.height;

    macro_rules! process {
        ($src_x_type:ty, $src_y_type:ty, $dest_x_type:ty, $dest_y_type:ty) => {
            let a_iter = SurfaceIter::<$src_x_type, $src_y_type>::new(a_surf_size, a_select_rect)?;
            let b_iter = SurfaceIter::<$dest_x_type, $dest_y_type>::new(b_surf_size, b_select_rect)?;
            for (a_idx, b_idx) in a_iter.zip(b_iter) {
                func(a_idx, b_idx);
            }
        };
    }

    // The following decision table avoids unnecessary wrapping calculations. We could use the `*Wrap` implementations everywhere, which
    // would also work, but is likely to be more expensive. Compare:
    // * Doing a modulo operation for every pixel (X-axis) and additionally for every row (Y-axis).
    // * Going through several `if`s that is required for the following table.
    //
    // NB: This table is generated by `test_module_fns::generate_surface_iterate_table()`.
    match (hflip, vflip, src_x_wrap, src_y_wrap, dest_x_wrap, dest_y_wrap) {
        (false, false, false, false, false, false) => { process!(Ascending, Ascending, Ascending, Ascending); }
        (false, false, false, false, false, true) => { process!(Ascending, Ascending, Ascending, AscendingWrap); }
        (false, false, false, false, true, false) => { process!(Ascending, Ascending, AscendingWrap, Ascending); }
        (false, false, false, false, true, true) => { process!(Ascending, Ascending, AscendingWrap, AscendingWrap); }
        (false, false, false, true, false, false) => { process!(Ascending, AscendingWrap, Ascending, Ascending); }
        (false, false, false, true, false, true) => { process!(Ascending, AscendingWrap, Ascending, AscendingWrap); }
        (false, false, false, true, true, false) => { process!(Ascending, AscendingWrap, AscendingWrap, Ascending); }
        (false, false, false, true, true, true) => { process!(Ascending, AscendingWrap, AscendingWrap, AscendingWrap); }
        (false, false, true, false, false, false) => { process!(AscendingWrap, Ascending, Ascending, Ascending); }
        (false, false, true, false, false, true) => { process!(AscendingWrap, Ascending, Ascending, AscendingWrap); }
        (false, false, true, false, true, false) => { process!(AscendingWrap, Ascending, AscendingWrap, Ascending); }
        (false, false, true, false, true, true) => { process!(AscendingWrap, Ascending, AscendingWrap, AscendingWrap); }
        (false, false, true, true, false, false) => { process!(AscendingWrap, AscendingWrap, Ascending, Ascending); }
        (false, false, true, true, false, true) => { process!(AscendingWrap, AscendingWrap, Ascending, AscendingWrap); }
        (false, false, true, true, true, false) => { process!(AscendingWrap, AscendingWrap, AscendingWrap, Ascending); }
        (false, false, true, true, true, true) => { process!(AscendingWrap, AscendingWrap, AscendingWrap, AscendingWrap); }
        (false, true, false, false, false, false) => { process!(Ascending, Descending, Ascending, Ascending); }
        (false, true, false, false, false, true) => { process!(Ascending, Descending, Ascending, AscendingWrap); }
        (false, true, false, false, true, false) => { process!(Ascending, Descending, AscendingWrap, Ascending); }
        (false, true, false, false, true, true) => { process!(Ascending, Descending, AscendingWrap, AscendingWrap); }
        (false, true, false, true, false, false) => { process!(Ascending, DescendingWrap, Ascending, Ascending); }
        (false, true, false, true, false, true) => { process!(Ascending, DescendingWrap, Ascending, AscendingWrap); }
        (false, true, false, true, true, false) => { process!(Ascending, DescendingWrap, AscendingWrap, Ascending); }
        (false, true, false, true, true, true) => { process!(Ascending, DescendingWrap, AscendingWrap, AscendingWrap); }
        (false, true, true, false, false, false) => { process!(AscendingWrap, Descending, Ascending, Ascending); }
        (false, true, true, false, false, true) => { process!(AscendingWrap, Descending, Ascending, AscendingWrap); }
        (false, true, true, false, true, false) => { process!(AscendingWrap, Descending, AscendingWrap, Ascending); }
        (false, true, true, false, true, true) => { process!(AscendingWrap, Descending, AscendingWrap, AscendingWrap); }
        (false, true, true, true, false, false) => { process!(AscendingWrap, DescendingWrap, Ascending, Ascending); }
        (false, true, true, true, false, true) => { process!(AscendingWrap, DescendingWrap, Ascending, AscendingWrap); }
        (false, true, true, true, true, false) => { process!(AscendingWrap, DescendingWrap, AscendingWrap, Ascending); }
        (false, true, true, true, true, true) => { process!(AscendingWrap, DescendingWrap, AscendingWrap, AscendingWrap); }
        (true, false, false, false, false, false) => { process!(Descending, Ascending, Ascending, Ascending); }
        (true, false, false, false, false, true) => { process!(Descending, Ascending, Ascending, AscendingWrap); }
        (true, false, false, false, true, false) => { process!(Descending, Ascending, AscendingWrap, Ascending); }
        (true, false, false, false, true, true) => { process!(Descending, Ascending, AscendingWrap, AscendingWrap); }
        (true, false, false, true, false, false) => { process!(Descending, AscendingWrap, Ascending, Ascending); }
        (true, false, false, true, false, true) => { process!(Descending, AscendingWrap, Ascending, AscendingWrap); }
        (true, false, false, true, true, false) => { process!(Descending, AscendingWrap, AscendingWrap, Ascending); }
        (true, false, false, true, true, true) => { process!(Descending, AscendingWrap, AscendingWrap, AscendingWrap); }
        (true, false, true, false, false, false) => { process!(DescendingWrap, Ascending, Ascending, Ascending); }
        (true, false, true, false, false, true) => { process!(DescendingWrap, Ascending, Ascending, AscendingWrap); }
        (true, false, true, false, true, false) => { process!(DescendingWrap, Ascending, AscendingWrap, Ascending); }
        (true, false, true, false, true, true) => { process!(DescendingWrap, Ascending, AscendingWrap, AscendingWrap); }
        (true, false, true, true, false, false) => { process!(DescendingWrap, AscendingWrap, Ascending, Ascending); }
        (true, false, true, true, false, true) => { process!(DescendingWrap, AscendingWrap, Ascending, AscendingWrap); }
        (true, false, true, true, true, false) => { process!(DescendingWrap, AscendingWrap, AscendingWrap, Ascending); }
        (true, false, true, true, true, true) => { process!(DescendingWrap, AscendingWrap, AscendingWrap, AscendingWrap); }
        (true, true, false, false, false, false) => { process!(Descending, Descending, Ascending, Ascending); }
        (true, true, false, false, false, true) => { process!(Descending, Descending, Ascending, AscendingWrap); }
        (true, true, false, false, true, false) => { process!(Descending, Descending, AscendingWrap, Ascending); }
        (true, true, false, false, true, true) => { process!(Descending, Descending, AscendingWrap, AscendingWrap); }
        (true, true, false, true, false, false) => { process!(Descending, DescendingWrap, Ascending, Ascending); }
        (true, true, false, true, false, true) => { process!(Descending, DescendingWrap, Ascending, AscendingWrap); }
        (true, true, false, true, true, false) => { process!(Descending, DescendingWrap, AscendingWrap, Ascending); }
        (true, true, false, true, true, true) => { process!(Descending, DescendingWrap, AscendingWrap, AscendingWrap); }
        (true, true, true, false, false, false) => { process!(DescendingWrap, Descending, Ascending, Ascending); }
        (true, true, true, false, false, true) => { process!(DescendingWrap, Descending, Ascending, AscendingWrap); }
        (true, true, true, false, true, false) => { process!(DescendingWrap, Descending, AscendingWrap, Ascending); }
        (true, true, true, false, true, true) => { process!(DescendingWrap, Descending, AscendingWrap, AscendingWrap); }
        (true, true, true, true, false, false) => { process!(DescendingWrap, DescendingWrap, Ascending, Ascending); }
        (true, true, true, true, false, true) => { process!(DescendingWrap, DescendingWrap, Ascending, AscendingWrap); }
        (true, true, true, true, true, false) => { process!(DescendingWrap, DescendingWrap, AscendingWrap, Ascending); }
        (true, true, true, true, true, true) => { process!(DescendingWrap, DescendingWrap, AscendingWrap, AscendingWrap); }
    }

    Ok(())
}

#[cfg(test)]
mod test_fn_surface_iterate_2 {
    use crate::geom::{Point, Rect, Size};
    use crate::sprite::GenericSurface;
    use super::Surface;
    use super::surface_iterate_2;

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

    fn create_source() -> Surfy {
        let mut src = Surfy::new(Size::new(12, 8), 0);
        assert_eq!(&EMPTY_DATA, src.data());

        src.data_mut().copy_from_slice(&SOURCE_DATA);
        assert_eq!(&SOURCE_DATA, src.data());
        src
    }

    macro_rules! source_spec {
        ($rect:expr, @hflip, @vflip) => {
            ($rect, true, true)
        };
        ($rect:expr, @hflip) => {
            ($rect, true, false)
        };
        ($rect:expr, @vflip) => {
            ($rect, false, true)
        };
        ($rect:expr) => {
            ($rect, false, false)
        };
    }

    fn copy_data(src_surf: &Surfy, dest_surf: &mut Surfy, (src_rect, hflip, vflip): (Rect, bool, bool), dest_point: Point) {
        let src_size = src_surf.size();
        let dest_size = dest_surf.size();

        let src = src_surf.data();
        let dest = dest_surf.data_mut();

        surface_iterate_2(src_size, src_rect, dest_size, dest_point, hflip, vflip,
                          |src_idx, dest_idx| {
                            dest[dest_idx] = src[src_idx];
                        },
        ).unwrap();
    }

    /// Test with a copy of the entire source surface without any flipping.
    #[test]
    fn test_full_copy_no_flip() {
        let src = create_source();
        let mut dest = Surfy::new(src.size(), 0);
        let src_spec = source_spec!(Rect::new((0, 0).into(), src.size()));
        let dest_point = (0, 0).into();
        copy_data(&src, &mut dest, src_spec, dest_point);

        assert_eq!(&SOURCE_DATA, dest.data());
    }

    /// Test with a copy of the entire source surface with horizontal flipping.
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

        let src = create_source();
        let mut dest = Surfy::new(src.size(), 0);
        let src_spec = source_spec!(Rect::new((0, 0).into(), src.size()), @hflip);
        let dest_point = (0, 0).into();
        copy_data(&src, &mut dest, src_spec, dest_point);

        assert_eq!(&EXPECTED, dest.data());
    }

    /// Test with a copy of the entire source surface with vertical flipping.
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

        let src = create_source();
        let mut dest = Surfy::new(src.size(), 0);
        let src_spec = source_spec!(Rect::new((0, 0).into(), src.size()), @vflip);
        let dest_point = (0, 0).into();
        copy_data(&src, &mut dest, src_spec, dest_point);

        assert_eq!(&EXPECTED, dest.data());
    }

    /// Test with a copy of the entire source surface with both horizontal and vertical flipping.
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

        let src = create_source();
        let mut dest = Surfy::new(src.size(), 0);
        let src_spec = source_spec!(Rect::new((0, 0).into(), src.size()), @hflip, @vflip);
        let dest_point = (0, 0).into();
        copy_data(&src, &mut dest, src_spec, dest_point);

        assert_eq!(&EXPECTED, dest.data());
    }

    /// Test with a copy of a partial surface with no wrapping.
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
            let src_spec = source_spec!(Rect::from(((1, 4), 4, 4)));
            let dest_point = (6, 3).into();
            copy_data(&src, &mut dest, src_spec, dest_point);

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
            let src_spec = source_spec!(Rect::from(((1, 4), 4, 4)), @hflip);
            let dest_point = (6, 3).into();
            copy_data(&src, &mut dest, src_spec, dest_point);

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
            let src_spec = source_spec!(Rect::from(((1, 4), 4, 4)), @vflip);
            let dest_point = (6, 3).into();
            copy_data(&src, &mut dest, src_spec, dest_point);

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
            let src_spec = source_spec!(Rect::from(((1, 4), 4, 4)), @hflip, @vflip);
            let dest_point = (6, 3).into();
            copy_data(&src, &mut dest, src_spec, dest_point);

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
            let src_spec = source_spec!(Rect::from(((10, 4), 4, 4)));
            let dest_point = (6, 3).into();
            copy_data(&src, &mut dest, src_spec, dest_point);

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
            let src_spec = source_spec!(Rect::from(((1, 4), 4, 4)));
            let dest_point = (10, 3).into();
            copy_data(&src, &mut dest, src_spec, dest_point);

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
            let src_spec = source_spec!(Rect::from(((1, 6), 4, 4)));
            let dest_point = (6, 3).into();
            copy_data(&src, &mut dest, src_spec, dest_point);

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
            let src_spec = source_spec!(Rect::from(((1, 4), 4, 4)));
            let dest_point = (6, 6).into();
            copy_data(&src, &mut dest, src_spec, dest_point);

            assert_eq!(&EXPECTED, dest.data());
        }
    }
    /// Test with a copy of a partial surface with both horizontal and vertical wrapping.
    #[test]
    fn test_partial_hv_wrap() {
        // H-wrap and v-wrap on src
        {
            const EXPECTED: [u8; 12 * 8] = data![
                0 0 0 0 0 0 0 0 0 0 0 0
                0 0 0 0 0 0 0 0 0 0 0 0
                0 0 0 0 0 0 0 0 0 0 0 0
                0 0 0 0 0 0 0 0 0 0 0 0
                0 0 0 0 0 0 0 0 0 0 0 0
                0 0 0 0 0 0 0 0 0 0 0 0
                0 0 0 0 0 0 1 0 0 0 0 0
                0 0 0 0 0 0 0 0 0 0 0 0
            ];

            let src = create_source();
            let mut dest = Surfy::new(src.size(), 0);
            let src_spec = source_spec!(Rect::from(((10, 6), 4, 4)));
            let dest_point = (6, 3).into();
            copy_data(&src, &mut dest, src_spec, dest_point);

            assert_eq!(&EXPECTED, dest.data());
        }

        // H-wrap and v-wrap on dest
        {
            const EXPECTED: [u8; 12 * 8] = data![
                3 3 0 0 0 0 0 0 0 0 0 0
                1 4 0 0 0 0 0 0 0 0 0 1
                0 0 0 0 0 0 0 0 0 0 0 0
                0 0 0 0 0 0 0 0 0 0 0 0
                0 0 0 0 0 0 0 0 0 0 0 0
                0 0 0 0 0 0 0 0 0 0 0 0
                2 2 0 0 0 0 0 0 0 0 2 3
                3 3 0 0 0 0 0 0 0 0 2 2
            ];

            let src = create_source();
            let mut dest = Surfy::new(src.size(), 0);
            let src_spec = source_spec!(Rect::from(((1, 4), 4, 4)));
            let dest_point = (10, 6).into();
            copy_data(&src, &mut dest, src_spec, dest_point);

            assert_eq!(&EXPECTED, dest.data());
        }
    }

    /// Function to generate decision table for `surface_iterate()`.
    //#[test]
    fn generate_surface_iterate_table() {
        const BOOLS: [bool; 2] = [false, true];

        fn direction(flip: bool) -> &'static str {
            if flip {
                "Descending"
            } else {
                "Ascending"
            }
        }

        fn wrapping(wrap: bool) -> &'static str {
            if wrap {
                "Wrap"
            } else {
                ""
            }
        }

        for hflip in BOOLS {
            for vflip in BOOLS {
                for src_x_wrap in BOOLS {
                    for src_y_wrap in BOOLS {
                        for dest_x_wrap in BOOLS {
                            for dest_y_wrap in BOOLS {
                                println!("({}, {}, {}, {}, {}, {}) => {{ process!({}{}, {}{}, {}{}, {}{}); }}",
                                         hflip, vflip, src_x_wrap, src_y_wrap, dest_x_wrap, dest_y_wrap,
                                         direction(hflip), wrapping(src_x_wrap),
                                         direction(vflip), wrapping(src_y_wrap),
                                         direction(false), wrapping(dest_x_wrap),
                                         direction(false), wrapping(dest_y_wrap),
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}