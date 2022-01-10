//! A module for working with 2-dimensional surfaces.

use std::ops::RangeInclusive;
use ves_geom::{SpaceUnit, Point, Rect, Size};

/// A 2-dimensional surface.
pub trait Surface<T: SpaceUnit> {
    type DataType;

    /// The size.
    fn size(&self) -> Size<T>;

    /// Retrieves a slice of the raw data.
    fn data(&self) -> &[Self::DataType];

    /// Retrieves a mutable slice of the raw data.
    fn data_mut(&mut self) -> &mut [Self::DataType];
}

/// A trait that allows for the retrieval of an offset for a certain value. This is useful for things like [`Surface`] implementations where
/// a coordinate can be translated to the offset in the raw data.
pub trait Offset {
    type Input;

    /// Returns the offset for the provided value.
    fn offset(&self, value: Self::Input) -> Option<usize>;
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
    pub fn new<T: SpaceUnit + Into<usize>>(size_surf: Size<T>, rect_view: Rect<T>) -> Result<Self, String> {
        let width: usize =  size_surf.width.into();
        let height: usize = size_surf.height.into();
        let x_min: usize = rect_view.min_x().into();
        let x_max: usize = rect_view.max_x().into();
        let x_iter = X::new_iter(x_min, x_max, width)
            .map_err(|msg| format!("Could not create iterator for X-axis (min: {}, max: {}, limit: {}): {}", x_min, x_max, width, msg))?;
        let y_min: usize = rect_view.min_y().into();
        let y_max: usize = rect_view.max_y().into();
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

/// Iterates over the indices for a selection in a pair of [`Surface`]s and passes the indices to the provided function.
///
/// # Parameters
/// * `a_surf_size`: The size of the first surface.
/// * `a_select_rect`: The selection rectangle in the first surface.
/// * `b_surf_size`: The size of the second surface.
/// * `b_select_origin`: The point of origin of the selection rectangle in the second surface. The selection rectangle will have the size of
///                      `a_select_rect`.
/// * `hflip`: A flag indicating that the iteration order on the horizontal axis should be inversed.
/// * `vflip`: A flag indicating that the iteration order on the vertical axis should be inversed.
/// * `func`: The function to call for every index.
///
/// # Returns
/// `Err` of a selection entirely exceeds a surface bound, otherwise `Ok`.
///
/// # Example
///
/// ```
/// use art_extractor_core::surface::surface_iterate_2;
/// use art_extractor_core::geom_art::{Size, Rect, Point};
///
/// let mut exp_iter: std::slice::Iter<(usize, usize)> = [
///     (22, 8080), (23, 8081), (24, 8082), (25, 8083), (32, 8180), (33, 8181), (34, 8182), (35, 8183),
///     (42, 8280), (43, 8281), (44, 8282), (45, 8283), (52, 8380), (53, 8381), (54, 8382), (55, 8383),
/// ].iter();
///
/// surface_iterate_2(
///     Size::new_raw(10, 10), // a_surf_size
///     Rect::new(Point::new_raw(2, 2), Size::new_raw(4, 4)), // a_select_rect
///     Size::new_raw(100, 100), // b_surf_size
///     Point::new_raw(80, 80), // b_select_origin
///     false, // hflip
///     false, // vflip
///     |idx_a, idx_b| { // func
///         let (exp_a, exp_b) = exp_iter.next().unwrap();
///         assert_eq!(*exp_a, idx_a);
///         assert_eq!(*exp_b, idx_b);
///     },
/// ).unwrap();
/// ```
pub fn surface_iterate<T: SpaceUnit + Into<usize>, F>(surf_size: Size<T>, select_rect: Rect<T>, hflip: bool, vflip: bool, func: F) -> Result<(), String> where
    F: FnMut(usize)
{
    let x_wrap = select_rect.max_x() >= surf_size.width;
    let y_wrap = select_rect.max_y() >= surf_size.height;

    macro_rules! process {
        ($x_type:ty, $y_type:ty) => {
            SurfaceIter::<$x_type, $y_type>::new(surf_size, select_rect)?
                .for_each(func);
        };
    }

    // The following decision table avoids unnecessary wrapping calculations. We could use the `*Wrap` implementations everywhere, which
    // would also work, but is likely to be more expensive. Compare:
    // * Doing a modulo operation for every pixel (X-axis) and additionally for every row (Y-axis).
    // * Going through several `if`s that is required for the following table.
    //
    // NB: This table is generated by `test_module_fns::generate_surface_iterate_table()`.
    match (hflip, vflip, x_wrap, y_wrap) {
        (false, false, false, false) => { process!(Ascending, Ascending); }
        (false, false, false, true) => { process!(Ascending, AscendingWrap); }
        (false, false, true, false) => { process!(AscendingWrap, Ascending); }
        (false, false, true, true) => { process!(AscendingWrap, AscendingWrap); }
        (false, true, false, false) => { process!(Ascending, Descending); }
        (false, true, false, true) => { process!(Ascending, DescendingWrap); }
        (false, true, true, false) => { process!(AscendingWrap, Descending); }
        (false, true, true, true) => { process!(AscendingWrap, DescendingWrap); }
        (true, false, false, false) => { process!(Descending, Ascending); }
        (true, false, false, true) => { process!(Descending, AscendingWrap); }
        (true, false, true, false) => { process!(DescendingWrap, Ascending); }
        (true, false, true, true) => { process!(DescendingWrap, AscendingWrap); }
        (true, true, false, false) => { process!(Descending, Descending); }
        (true, true, false, true) => { process!(Descending, DescendingWrap); }
        (true, true, true, false) => { process!(DescendingWrap, Descending); }
        (true, true, true, true) => { process!(DescendingWrap, DescendingWrap); }
    }

    Ok(())
}

#[cfg(test)]
mod test_fn_surface_iterate {
    /// Function to generate decision table for `surface_iterate()`.
    // #[test]
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
                for x_wrap in BOOLS {
                    for y_wrap in BOOLS {
                        println!("({}, {}, {}, {}) => {{ process!({}{}, {}{}); }}",
                                 hflip, vflip, x_wrap, y_wrap,
                                 direction(hflip), wrapping(x_wrap),
                                 direction(vflip), wrapping(y_wrap),
                        );
                    }
                }
            }
        }
    }
}

/// Iterates over the indices for a selection in a pair of [`Surface`]s and passes the indices to the provided function.
///
/// # Parameters
/// * `a_surf_size`: The size of the first surface.
/// * `a_select_rect`: The selection rectangle in the first surface.
/// * `b_surf_size`: The size of the second surface.
/// * `b_select_origin`: The point of origin of the selection rectangle in the second surface. The selection rectangle will have the size of
///                      `a_select_rect`.
/// * `hflip`: A flag indicating that the iteration order on the horizontal axis should be inversed.
/// * `vflip`: A flag indicating that the iteration order on the vertical axis should be inversed.
/// * `func`: The function to call for every index.
///
/// # Returns
/// `Err` of a selection entirely exceeds a surface bound, otherwise `Ok`.
///
/// # Example
///
/// ```
/// use art_extractor_core::surface::surface_iterate_2;
/// use art_extractor_core::geom_art::{Size, Rect, Point};
///
/// let mut exp_iter: std::slice::Iter<(usize, usize)> = [
///     (22, 8080), (23, 8081), (24, 8082), (25, 8083), (32, 8180), (33, 8181), (34, 8182), (35, 8183),
///     (42, 8280), (43, 8281), (44, 8282), (45, 8283), (52, 8380), (53, 8381), (54, 8382), (55, 8383),
/// ].iter();
///
/// surface_iterate_2(
///     Size::new_raw(10, 10), // a_surf_size
///     Rect::new(Point::new_raw(2, 2), Size::new_raw(4, 4)), // a_select_rect
///     Size::new_raw(100, 100), // b_surf_size
///     Point::new_raw(80, 80), // b_select_origin
///     false, // hflip
///     false, // vflip
///     |idx_a, idx_b| { // func
///         let (exp_a, exp_b) = exp_iter.next().unwrap();
///         assert_eq!(*exp_a, idx_a);
///         assert_eq!(*exp_b, idx_b);
///     },
/// ).unwrap();
/// ```
pub fn surface_iterate_2<T: SpaceUnit + Into<usize>, F>(a_surf_size: Size<T>, a_select_rect: Rect<T>, b_surf_size: Size<T>, b_select_origin: Point<T>, hflip: bool, vflip: bool, mut func: F) -> Result<(), String> where
    F: FnMut(usize, usize)
{
    let b_select_rect = Rect::<T>::new(b_select_origin, a_select_rect.size);
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
    // NB: This table is generated by `test_module_fns::generate_surface_iterate_2_table()`.
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
mod test_fn_surface_iterate_2;

#[cfg(test)]
mod test_surface_iter;
