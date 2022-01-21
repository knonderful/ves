//! Unit tests for `SurfaceIter`.

use crate::geom_art::{ArtworkSpaceUnit, Rect};
use crate::surface::Surface;

crate::sized_surface!(Surfy, u8, ArtworkSpaceUnit, 12, 8, 0);

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
            $crate::surface::SurfaceIter::<ArtworkSpaceUnit, $crate::surface::DescendingWrap, $crate::surface::DescendingWrap>::new($size, $rect).unwrap().map(|tuple| tuple.1)
        };
        ($size:expr, $rect:expr, @hflip) => {
            $crate::surface::SurfaceIter::<ArtworkSpaceUnit, $crate::surface::DescendingWrap, $crate::surface::AscendingWrap>::new($size, $rect).unwrap().map(|tuple| tuple.1)
        };
        ($size:expr, $rect:expr, @vflip) => {
            $crate::surface::SurfaceIter::<ArtworkSpaceUnit, $crate::surface::AscendingWrap, $crate::surface::DescendingWrap>::new($size, $rect).unwrap().map(|tuple| tuple.1)
        };
        ($size:expr, $rect:expr) => {
            $crate::surface::SurfaceIter::<ArtworkSpaceUnit, $crate::surface::AscendingWrap, $crate::surface::AscendingWrap>::new($size, $rect).unwrap().map(|tuple| tuple.1)
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
    let mut src = Surfy::new();
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
        let mut dest = Surfy::new();
        let src_iter = surface_iter!(src.size(), Rect::new_from_size((0, 0).into(), src.size()));
        let dest_iter = surface_iter!(dest.size(), Rect::new_from_size((0, 0).into(), dest.size()));
        copy_data(&src, &mut dest, src_iter, dest_iter);
        assert_eq!(&SOURCE_DATA, dest.data());
    }

    // H-flip on both
    {
        let src = create_source();
        let mut dest = Surfy::new();
        let src_iter = surface_iter!(src.size(), Rect::new_from_size((0, 0).into(), src.size()), @hflip);
        let dest_iter = surface_iter!(dest.size(), Rect::new_from_size((0, 0).into(), dest.size()), @hflip);
        copy_data(&src, &mut dest, src_iter, dest_iter);
        assert_eq!(&SOURCE_DATA, dest.data());
    }

    // V-flip on both
    {
        let src = create_source();
        let mut dest = Surfy::new();
        let src_iter = surface_iter!(src.size(), Rect::new_from_size((0, 0).into(), src.size()), @vflip);
        let dest_iter = surface_iter!(dest.size(), Rect::new_from_size((0, 0).into(), dest.size()), @vflip);
        copy_data(&src, &mut dest, src_iter, dest_iter);
        assert_eq!(&SOURCE_DATA, dest.data());
    }

    // H-flip and v-flip on both
    {
        let src = create_source();
        let mut dest = Surfy::new();
        let src_iter = surface_iter!(src.size(), Rect::new_from_size((0, 0).into(), src.size()), @hflip, @vflip);
        let dest_iter = surface_iter!(dest.size(), Rect::new_from_size((0, 0).into(), dest.size()), @hflip, @vflip);
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
        let mut dest = Surfy::new();
        let src_iter = surface_iter!(src.size(), Rect::new_from_size((0, 0).into(), src.size()), @hflip);
        let dest_iter = surface_iter!(dest.size(), Rect::new_from_size((0, 0).into(), dest.size()));
        copy_data(&src, &mut dest, src_iter, dest_iter);
        assert_eq!(&EXPECTED, dest.data());
    }

    // H-flip on dest
    {
        let src = create_source();
        let mut dest = Surfy::new();
        let src_iter = surface_iter!(src.size(), Rect::new_from_size((0, 0).into(), src.size()));
        let dest_iter = surface_iter!(dest.size(), Rect::new_from_size((0, 0).into(), dest.size()), @hflip);
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
        let mut dest = Surfy::new();
        let src_iter = surface_iter!(src.size(), Rect::new_from_size((0, 0).into(), src.size()), @vflip);
        let dest_iter = surface_iter!(dest.size(), Rect::new_from_size((0, 0).into(), dest.size()));
        copy_data(&src, &mut dest, src_iter, dest_iter);
        assert_eq!(&EXPECTED, dest.data());
    }

    // V-flip on dest
    {
        let src = create_source();
        let mut dest = Surfy::new();
        let src_iter = surface_iter!(src.size(), Rect::new_from_size((0, 0).into(), src.size()));
        let dest_iter = surface_iter!(dest.size(), Rect::new_from_size((0, 0).into(), dest.size()), @vflip);
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
        let mut dest = Surfy::new();
        let src_iter = surface_iter!(src.size(), Rect::new_from_size((0, 0).into(), src.size()), @hflip, @vflip);
        let dest_iter = surface_iter!(dest.size(), Rect::new_from_size((0, 0).into(), dest.size()));
        copy_data(&src, &mut dest, src_iter, dest_iter);
        assert_eq!(&EXPECTED, dest.data());
    }

    // H-flip and v-flip on dest
    {
        let src = create_source();
        let mut dest = Surfy::new();
        let src_iter = surface_iter!(src.size(), Rect::new_from_size((0, 0).into(), src.size()));
        let dest_iter = surface_iter!(dest.size(), Rect::new_from_size((0, 0).into(), dest.size()), @hflip, @vflip);
        copy_data(&src, &mut dest, src_iter, dest_iter);
        assert_eq!(&EXPECTED, dest.data());
    }

    // H-flip on src and v-flip on dest
    {
        let src = create_source();
        let mut dest = Surfy::new();
        let src_iter = surface_iter!(src.size(), Rect::new_from_size((0, 0).into(), src.size()), @hflip);
        let dest_iter = surface_iter!(dest.size(), Rect::new_from_size((0, 0).into(), dest.size()), @vflip);
        copy_data(&src, &mut dest, src_iter, dest_iter);
        assert_eq!(&EXPECTED, dest.data());
    }

    // H-flip on dest and v-flip on src
    {
        let src = create_source();
        let mut dest = Surfy::new();
        let src_iter = surface_iter!(src.size(), Rect::new_from_size((0, 0).into(), src.size()), @vflip);
        let dest_iter = surface_iter!(dest.size(), Rect::new_from_size((0, 0).into(), dest.size()), @hflip);
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
        let mut dest = Surfy::new();
        let src_iter = surface_iter!(src.size(), ((1, 4), (4, 7)).into());
        let dest_iter = surface_iter!(dest.size(), ((6, 3), (9, 6)).into());
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
        let mut dest = Surfy::new();
        let src_iter = surface_iter!(src.size(), ((1, 4), (4, 7)).into(), @hflip);
        let dest_iter = surface_iter!(dest.size(), ((6, 3), (9, 6)).into());
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
        let mut dest = Surfy::new();
        let src_iter = surface_iter!(src.size(), ((1, 4), (4, 7)).into(), @vflip);
        let dest_iter = surface_iter!(dest.size(), ((6, 3), (9, 6)).into());
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
        let mut dest = Surfy::new();
        let src_iter = surface_iter!(src.size(), ((1, 4), (4, 7)).into(), @hflip, @vflip);
        let dest_iter = surface_iter!(dest.size(), ((6, 3), (9, 6)).into());
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
        let mut dest = Surfy::new();
        let src_iter = surface_iter!(src.size(), ((10, 4), (13, 7)).into());
        let dest_iter = surface_iter!(dest.size(), ((6, 3), (9, 6)).into());
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
        let mut dest = Surfy::new();
        let src_iter = surface_iter!(src.size(), ((1, 4), (4, 7)).into());
        let dest_iter = surface_iter!(dest.size(), ((10, 3), (13, 6)).into());
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
        let mut dest = Surfy::new();
        let src_iter = surface_iter!(src.size(), ((1, 6), (4, 9)).into());
        let dest_iter = surface_iter!(dest.size(), ((6, 3), (9, 6)).into());
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
        let mut dest = Surfy::new();
        let src_iter = surface_iter!(src.size(), ((1, 4), (4, 7)).into());
        let dest_iter = surface_iter!(dest.size(), ((6, 6), (9, 9)).into());
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
        let mut dest = Surfy::new();
        let src_iter = surface_iter!(src.size(), ((10, 6), (13, 9)).into());
        let dest_iter = surface_iter!(dest.size(), ((6, 3), (9, 6)).into());
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
        let mut dest = Surfy::new();
        let src_iter = surface_iter!(src.size(), ((1, 4), (4, 7)).into());
        let dest_iter = surface_iter!(dest.size(), ((10, 6), (13, 9)).into());
        copy_data(&src, &mut dest, src_iter, dest_iter);
        assert_eq!(&EXPECTED, dest.data());
    }
}
