use crate::egui;

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
    } else {
        if vflip {
            egui::Rect::from_min_max(
                egui::pos2(rect.min.x, rect.max.y),
                egui::pos2(rect.max.x, rect.min.y),
            )
        } else {
            rect
        }
    }
}

pub struct Sprite {
    pub rect: art_extractor_core::geom_art::Rect,
    pub texture: egui::TextureHandle,
    pub hflip: bool,
    pub vflip: bool,
}

impl Sprite {
    /// Create an [`egui::Image`] from this [`Sprite`].
    ///
    /// # Arguments
    ///
    /// * `size`: The size for the output image.
    ///
    /// returns: An [`egui::Image`].
    pub fn to_image(&self, size: egui::Vec2) -> egui::Image {
        egui::Image::new(&self.texture, size).uv(correct_uv(
            DEFAULT_UV,
            self.hflip,
            self.vflip,
        ))
    }

    /// Calculates the UV [`egui::Rect`] for a section of this [`Sprite`].
    ///
    /// # Arguments
    ///
    /// * `rect`: The rectangle that defines the section.
    ///
    /// returns: A [`egui::Rect`] that represents the UV values of the underlying texture.
    pub fn partial_uv(&self, rect: &art_extractor_core::geom_art::Rect) -> egui::Rect {
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
}
