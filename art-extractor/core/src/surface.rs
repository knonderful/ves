//! A module for working with 2-dimensional surfaces.

use std::ops::Range;
use generator::done;
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

pub fn pixel_iterator(rect: Rect, hflip: bool, vflip: bool) -> impl Iterator<Item=Point> {
    macro_rules! create_iterator {
        ($x_method:expr, $y_method:expr) => {
            generator::Gn::new_scoped_local(move |mut scope| {
                for y in $y_method {
                    for x in $x_method {
                        scope.yield_(Point::new(x, y));
                    }
                }
                done!();
            })
        }
    }

    match (hflip, vflip) {
        (false, false) => create_iterator!(rect.range_x(), rect.range_y()),
        (true, false) => create_iterator!(rect.range_x().rev(), rect.range_y()),
        (false, true) => create_iterator!(rect.range_x(), rect.range_y().rev()),
        (true, true) => create_iterator!(rect.range_x().rev(), rect.range_y().rev()),
    }
}

#[cfg(test)]
mod test_module_functions {
    use crate::geom::generic::Point;
    use crate::surface::pixel_iterator;

    #[test]
    fn test_pixel_iterator() {
        let rect = ((8, 16), 4, 10).into();

        let mut iter = pixel_iterator(rect, false, false);
        for y in rect.range_y() {
            for x in rect.range_x() {
                assert_eq!(Point::new(x, y), iter.next().unwrap());
            }
        }
        assert_eq!(None, iter.next());

        let mut iter = pixel_iterator(rect, true, false);
        for y in rect.range_y() {
            for x in rect.range_x().rev() {
                assert_eq!(Point::new(x, y), iter.next().unwrap());
            }
        }
        assert_eq!(None, iter.next());

        let mut iter = pixel_iterator(rect, false, true);
        for y in rect.range_y().rev() {
            for x in rect.range_x() {
                assert_eq!(Point::new(x, y), iter.next().unwrap());
            }
        }
        assert_eq!(None, iter.next());

        let mut iter = pixel_iterator(rect, true, true);
        for y in rect.range_y().rev() {
            for x in rect.range_x().rev() {
                assert_eq!(Point::new(x, y), iter.next().unwrap());
            }
        }
        assert_eq!(None, iter.next());
    }
pub struct PixelIter<X, Y> {
    x_iter: X,
    y_iter: Y,
    current_x_iter: X,
    current_y: ArtworkSpaceUnit,
}

impl<X, Y> PixelIter<X, Y> where
    X: Iterator<Item=ArtworkSpaceUnit> + Clone,
    Y: Iterator<Item=ArtworkSpaceUnit>,
{
    pub fn new(x_iter: X, mut y_iter: Y) -> Self {
        let current_x_iter = x_iter.clone();
        let current_y = y_iter.next().expect("");
        Self { x_iter, y_iter, current_x_iter, current_y }
    }

    #[inline(always)]
    fn do_next(&mut self) -> Option<Point> {
        match self.current_x_iter.next() {
            Some(x) => Some(Point::new(x, self.current_y)),
            None => {
                match self.y_iter.next() {
                    None => None,
                    Some(y) => {
                        self.current_y = y;
                        self.current_x_iter = self.x_iter.clone();
                        self.do_next()
                    }
                }
            },
        }
    }
}

impl<X, Y> Iterator for PixelIter<X, Y> where
    X: Iterator<Item=ArtworkSpaceUnit> + Clone,
    Y: Iterator<Item=ArtworkSpaceUnit>,
{
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        self.do_next()
    }
}

#[cfg(test)]
mod test_pixel_iter {
    use crate::surface::PixelIter;
    use crate::geom::Rect;
    use crate::geom::{Point, Size};

    #[test]
    fn test_pixel_iter() {
        let rect = Rect::new((8, 16).into(), Size::new(4, 10));

        let mut iter = PixelIter::new(rect.range_x(), rect.range_y());
        for y in rect.range_y() {
            for x in rect.range_x() {
                assert_eq!(Point::new(x, y), iter.next().unwrap());
            }
        }
        assert_eq!(None, iter.next());

        let mut iter = PixelIter::new(rect.range_x().rev(), rect.range_y());
        for y in rect.range_y() {
            for x in rect.range_x().rev() {
                assert_eq!(Point::new(x, y), iter.next().unwrap());
            }
        }
        assert_eq!(None, iter.next());

        let mut iter = PixelIter::new(rect.range_x(), rect.range_y().rev());
        for y in rect.range_y().rev() {
            for x in rect.range_x() {
                assert_eq!(Point::new(x, y), iter.next().unwrap());
            }
        }
        assert_eq!(None, iter.next());

        let mut iter = PixelIter::new(rect.range_x().rev(), rect.range_y().rev());
        for y in rect.range_y().rev() {
            for x in rect.range_x().rev() {
                assert_eq!(Point::new(x, y), iter.next().unwrap());
            }
        }
        assert_eq!(None, iter.next());
    }

}