use art_extractor_core::geom_art::{ArtworkSpaceUnit, Point, Rect, Size};
use art_extractor_core::movie::MovieFrame;
use art_extractor_core::sprite::{Color, Palette, PaletteRef, Tile, TileRef};
use art_extractor_core::surface::{surface_iterate, Surface};
use bmp::Pixel;
use std::ops::Index;

art_extractor_core::sized_surface!(
    ScreenSurface,
    Color,
    ArtworkSpaceUnit,
    512,
    256,
    Color::Transparent
);

pub fn create_bitmap(
    size: Size,
    mut func: impl FnMut(usize, Point, &mut bmp::Image),
) -> bmp::Image {
    let mut img = bmp::Image::new(size.width.raw(), size.height.raw());

    let rect = size.as_rect();
    let mut pos_iter = (0..rect.height().raw())
        .flat_map(|y| std::iter::repeat(y).zip(0..rect.width().raw()))
        .map(|(y, x)| (x, y));

    surface_iterate(size, rect, false, false, |_pos, index| {
        let (x, y) = pos_iter.next().unwrap();
        func(index, Point::new(x, y), &mut img);
    })
    .unwrap();
    img
}

pub fn bmp_from_movie_frame(
    movie_frame: &MovieFrame,
    palettes: &impl Index<PaletteRef, Output = Palette>,
    tiles: &impl Index<TileRef, Output = Tile>,
) -> bmp::Image where {
    // Render everything to our special screen surface.
    let mut screen_surface = ScreenSurface::new();
    let screen_size = screen_surface.size();
    let screen_data = screen_surface.data_mut();

    // Reverse-iterate because the first objects should be rendered on top
    for sprite in movie_frame.sprites().iter().rev() {
        let tile = &tiles[sprite.tile()];
        let sprite_surface = tile.surface();
        let src_data = sprite_surface.data();
        let src_size = sprite_surface.size();
        let src_rect = Rect::new_from_size((0, 0), src_size);

        let palette = &palettes[sprite.palette()];
        art_extractor_core::surface::surface_iterate_2(
            src_size,
            src_rect,
            screen_size,
            sprite.position(),
            sprite.h_flip(),
            sprite.v_flip(),
            |_src_pos, src_idx, _dest_pos, dest_idx| {
                let index = src_data[src_idx];
                if index.value() == 0 {
                    return;
                }
                let color = palette[index];
                screen_data[dest_idx] = color;
            },
        )
        .unwrap();
    }

    // Write BMP
    let transparent = Pixel::new(255, 0, 255);
    super::test_util::create_bitmap(screen_size, |index, pos, img| {
        let color = screen_data[index];
        match color {
            Color::Opaque(color) => {
                img.set_pixel(
                    pos.x.raw(),
                    pos.y.raw(),
                    Pixel::new(color.r, color.g, color.b),
                );
            }
            Color::Transparent => {
                img.set_pixel(pos.x.raw(), pos.y.raw(), transparent);
            }
        }
    })
}
