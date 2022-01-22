//! Unit tests for `surface_iterate_2()`.

use crate::geom_art::{ArtworkSpaceUnit, Point, Rect};
use super::Surface;
use super::surface_iterate_2;

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

fn create_source() -> Surfy {
    let mut src = Surfy::new();
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
                      |_src_pos, src_idx, _dest_pos, dest_idx| {
                          dest[dest_idx] = src[src_idx];
                      },
    ).unwrap();
}

/// Test with a copy of the entire source surface without any flipping.
#[test]
fn test_full_copy_no_flip() {
    let src = create_source();
    let mut dest = Surfy::new();
    let src_spec = source_spec!(Rect::new_from_size((0, 0), src.size()));
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
    let mut dest = Surfy::new();
    let src_spec = source_spec!(Rect::new_from_size((0, 0), src.size()), @hflip);
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
    let mut dest = Surfy::new();
    let src_spec = source_spec!(Rect::new_from_size((0, 0), src.size()), @vflip);
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
    let mut dest = Surfy::new();
    let src_spec = source_spec!(Rect::new_from_size((0, 0), src.size()), @hflip, @vflip);
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
        let mut dest = Surfy::new();
        let src_spec = source_spec!(Rect::from(((1, 4), (4, 7))));
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
        let mut dest = Surfy::new();
        let src_spec = source_spec!(Rect::from(((1, 4), (4, 7))), @hflip);
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
        let mut dest = Surfy::new();
        let src_spec = source_spec!(Rect::from(((1, 4), (4, 7))), @vflip);
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
        let mut dest = Surfy::new();
        let src_spec = source_spec!(Rect::from(((1, 4), (4, 7))), @hflip, @vflip);
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
        let mut dest = Surfy::new();
        let src_spec = source_spec!(Rect::from(((10, 4), (13, 7))));
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
        let mut dest = Surfy::new();
        let src_spec = source_spec!(Rect::from(((1, 4), (4, 7))));
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
        let mut dest = Surfy::new();
        let src_spec = source_spec!(Rect::from(((1, 6), (4, 9))));
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
        let mut dest = Surfy::new();
        let src_spec = source_spec!(Rect::from(((1, 4), (4, 7))));
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
        let mut dest = Surfy::new();
        let src_spec = source_spec!(Rect::from(((10, 6), (13, 9))));
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
        let mut dest = Surfy::new();
        let src_spec = source_spec!(Rect::from(((1, 4), (4, 7))));
        let dest_point = (10, 6).into();
        copy_data(&src, &mut dest, src_spec, dest_point);

        assert_eq!(&EXPECTED, dest.data());
    }
}

/// Function to generate decision table for `surface_iterate_2()`.
//#[test]
fn generate_surface_iterate_2_table() {
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
            for a_x_wrap in BOOLS {
                for a_y_wrap in BOOLS {
                    for b_x_wrap in BOOLS {
                        for b_y_wrap in BOOLS {
                            println!("({}, {}, {}, {}, {}, {}) => {{ process!({}{}, {}{}, {}{}, {}{}); }}",
                                     hflip, vflip, a_x_wrap, a_y_wrap, b_x_wrap, b_y_wrap,
                                     direction(hflip), wrapping(a_x_wrap),
                                     direction(vflip), wrapping(a_y_wrap),
                                     direction(false), wrapping(b_x_wrap),
                                     direction(false), wrapping(b_y_wrap),
                            );
                        }
                    }
                }
            }
        }
    }
}
