//! A module for working with 2-dimensional surfaces.

use std::ops::Range;
use crate::geom::{ArtworkSpaceUnit, Rect, Size};

/// Local trait for extending `ArtworkSpaceUnit` with `into_usize()`.
trait IntoUsize {
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
