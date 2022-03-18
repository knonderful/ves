use crate::egui;
use std::ops::Index;
use ves_art_core::surface::Surface;

pub const DEFAULT_UV: egui::Rect =
    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));

fn correct_uv(rect: egui::Rect, hflip: bool, vflip: bool) -> egui::Rect {
    if hflip {
        if vflip {
            egui::Rect::from_min_max(rect.max, rect.min)
        } else {
            egui::Rect::from_min_max(
                egui::pos2(rect.max.x, rect.min.y),
                egui::pos2(rect.min.x, rect.max.y),
            )
        }
    } else if vflip {
        egui::Rect::from_min_max(
            egui::pos2(rect.min.x, rect.max.y),
            egui::pos2(rect.max.x, rect.min.y),
        )
    } else {
        rect
    }
}

pub struct Sprite {
    rect: ves_art_core::geom_art::Rect,
    texture: egui::TextureHandle,
    hflip: bool,
    vflip: bool,
}

impl Sprite {
    /// Creates a [`Sprite`] from a [`art_extractor_core::sprite::Sprite`].
    ///
    /// # Arguments
    ///
    /// * `sprite`: The source sprite.
    /// * `palettes`: The palettes.
    /// * `tiles`: The tiles.
    /// * `texture_factory`: A function for creating texture handles from image data.
    ///
    /// returns: The [`Sprite`].
    pub fn create(
        sprite: &ves_art_core::sprite::Sprite,
        palettes: &impl Index<ves_art_core::sprite::PaletteRef, Output = ves_art_core::sprite::Palette>,
        tiles: &impl Index<ves_art_core::sprite::TileRef, Output = ves_art_core::sprite::Tile>,
        mut texture_factory: impl FnMut(egui::ColorImage) -> egui::TextureHandle,
    ) -> Self {
        let palette = &palettes[sprite.palette()];
        let tile = &tiles[sprite.tile()];
        let color_image = Self::create_color_image(palette, tile);

        let texture = texture_factory(color_image);
        let rect =
            ves_art_core::geom_art::Rect::new_from_size(sprite.position(), tile.surface().size());

        Self {
            rect,
            texture,
            hflip: sprite.h_flip(),
            vflip: sprite.v_flip(),
        }
    }

    /// Retrieves the [`TextureHandle`](egui::TextureHandle).
    pub fn texture(&self) -> &egui::TextureHandle {
        &self.texture
    }

    /// Retrieves the [`Rect`](ves_art_core::geom_art::Rect).
    pub fn rect(&self) -> ves_art_core::geom_art::Rect {
        self.rect
    }

    /// Retrieves the horizontal flipping flag.
    #[allow(unused)]
    pub fn hflip(&self) -> bool {
        self.hflip
    }

    /// Retrieves the vertical flipping flag.
    #[allow(unused)]
    pub fn vflip(&self) -> bool {
        self.vflip
    }

    /// Create an [`egui::Image`] from this [`Sprite`].
    ///
    /// # Arguments
    ///
    /// * `size`: The size for the output image.
    ///
    /// returns: An [`egui::Image`].
    pub fn to_image(&self, size: egui::Vec2) -> egui::Image {
        egui::Image::new(&self.texture, size).uv(correct_uv(DEFAULT_UV, self.hflip, self.vflip))
    }

    /// Calculates the UV [`egui::Rect`] for a section of this [`Sprite`].
    ///
    /// # Arguments
    ///
    /// * `rect`: The rectangle that defines the section.
    ///
    /// returns: A [`egui::Rect`] that represents the UV values of the underlying texture.
    pub fn partial_uv(&self, rect: &ves_art_core::geom_art::Rect) -> egui::Rect {
        use crate::ToEgui as _;

        let egui_sprite_rect = self.rect.to_egui();
        let egui_rect = rect.to_egui();
        let width = egui_sprite_rect.width();
        let height = egui_sprite_rect.height();
        let mut u_x = (egui_rect.min.x - egui_sprite_rect.min.x) / width;
        let mut u_y = (egui_rect.min.y - egui_sprite_rect.min.y) / height;
        let mut v_x = (egui_rect.max.x - egui_sprite_rect.min.x) / width;
        let mut v_y = (egui_rect.max.y - egui_sprite_rect.min.y) / height;

        if self.hflip {
            u_x = 1.0 - u_x;
            v_x = 1.0 - v_x;
        }
        if self.vflip {
            u_y = 1.0 - u_y;
            v_y = 1.0 - v_y;
        }

        egui::Rect::from_min_max(egui::pos2(u_x, u_y), egui::pos2(v_x, v_y))
    }

    fn create_color_image(
        palette: &ves_art_core::sprite::Palette,
        tile: &ves_art_core::sprite::Tile,
    ) -> egui::ColorImage {
        let surf = tile.surface();
        let surf_data = surf.data();

        let mut raw_image = vec![0u8; surf.data().len() * 4]; // 4 bytes per pixel (RGBA)
        let mut raw_image_idx: usize = 0;

        ves_art_core::surface::surface_iterate(
            surf.size(),
            surf.size().as_rect(),
            false, // We do flipping in the mesh/Image instead of in the texture (using UV)
            false,
            |_, idx| {
                let color = &palette[surf_data[idx]];

                let col_data = match color {
                    ves_art_core::sprite::Color::Opaque(col) => [col.r, col.g, col.b, 0xff],
                    ves_art_core::sprite::Color::Transparent => [0x00, 0x00, 0x00, 0x00],
                };

                raw_image[raw_image_idx..raw_image_idx + 4].copy_from_slice(&col_data);
                raw_image_idx += 4;
            },
        )
        .unwrap();

        let w: usize = surf.size().width.raw().try_into().unwrap();
        let h: usize = surf.size().height.raw().try_into().unwrap();
        egui::ColorImage::from_rgba_unmultiplied([w, h], &raw_image)
    }
}
