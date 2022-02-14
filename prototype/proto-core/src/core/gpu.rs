use crate::gfx::{
    Rectangle2D, RectangleIterator, Rgb888, Rgba8888, SliceBackedSurface, SliceBackedSurfaceMut,
    Surface, SurfaceValueGet, SurfaceValueSet, Unit2D,
};
use proto_common::gpu::{OamTableEntry, OamTableIndex, OcmTableIndex};

/// The width of a character in pixels.
const CHAR_WIDTH: Unit2D = 8;
/// The height of a character in pixels.
const CHAR_HEIGHT: Unit2D = 8;
/// The width of the character table in number of characters.
const OBJ_CHAR_TABLE_WIDTH: Unit2D = 16;
/// The height of the character table in number of characters.
const OBJ_CHAR_TABLE_HEIGHT: Unit2D = 16;
/// The size of the object attribute table in number of entries.
const OBJ_ATTR_MEM_SIZE: usize = 32usize;

// TODO: Replace FrameBufferPixel with another pixel type that only stores the NECESSARY data (basically the indices, not the RGBA)
crate::linear_pixel_buffer!(
    OcmSurfaceBuffer,
    Rgb888,
    OBJ_CHAR_TABLE_WIDTH,
    OBJ_CHAR_TABLE_HEIGHT
);

/// A character table.
#[derive(Default)]
pub struct OcmTable {
    surface_buffer: OcmSurfaceBuffer,
}

impl OcmTable {
    /// Loads graphics data into the table.
    ///
    /// This method checks the passed `data` slice for the expected length. An invalid length will result in a panic.
    ///
    /// # Parameters
    /// * `index`: The [`OcmTableIndex`].
    /// * `data`: The graphics data.
    pub fn load(&mut self, index: OcmTableIndex, data: &[u8]) {
        let x = index.x() as Unit2D * CHAR_WIDTH;
        let y = index.y() as Unit2D * CHAR_HEIGHT;

        assert_eq!(data.len() as u32, CHAR_WIDTH * CHAR_HEIGHT * 3); // 3 bytes per pixel

        // TODO: Support different sized sprites here
        let src_surf = SliceBackedSurface::<Rgb888>::new(data, (CHAR_WIDTH, CHAR_HEIGHT).into());

        let mut dest_surf = self.surface_buffer.as_surface_mut();

        let src_iter = RectangleIterator::new(src_surf.dimensions());
        let dest_rect = Rectangle2D::new((x, y).into(), src_surf.dimensions());
        let dest_iter = RectangleIterator::new_with_rectangle(dest_surf.dimensions(), dest_rect);

        src_iter.zip(dest_iter).for_each(|(src_pos, dest_pos)| {
            dest_surf.set_value(dest_pos, &src_surf.get_value(src_pos));
        });
    }

    /// Retrieves the [`Surface`].
    pub fn surface(&self) -> SliceBackedSurface<Rgb888> {
        self.surface_buffer.as_surface()
    }

    /// Retrieves the [`Rectangle2D`] for the provided [`OamTableEntry`] in the surface of this OCM instance.
    ///
    /// # Parameters
    /// * `oam_entry`: The [`OamTableEntry`] that describes the object.
    pub fn obj_rectangle(&self, oam_entry: &OamTableEntry) -> Rectangle2D {
        let char_table_index = oam_entry.char_table_index();
        let origin = (
            char_table_index.x() as Unit2D * CHAR_WIDTH,
            char_table_index.y() as Unit2D * CHAR_HEIGHT,
        )
            .into();
        // TODO: Support different sized sprites here
        Rectangle2D::new(origin, (CHAR_WIDTH, CHAR_HEIGHT).into())
    }
}

#[derive(Default)]
pub struct OamTable {
    data: [Option<OamTableEntry>; OBJ_ATTR_MEM_SIZE],
}

impl OamTable {
    /// Sets an entry.
    ///
    /// # Parameters
    /// * `index`: The [`OamTableIndex`].
    /// * `entry`: The [`OamTableEntry`].
    pub fn set(&mut self, index: OamTableIndex, entry: OamTableEntry) {
        self.data[u8::from(index) as usize] = Some(entry)
    }

    /// Renders the objects.
    ///
    /// # Parameters
    /// * `ocm_table`: The [`OcmTable`] that contains the graphics data.
    /// * `surface`: The target surface.
    pub fn render(&self, ocm_table: &OcmTable, surface: &mut SliceBackedSurfaceMut<Rgba8888>) {
        // Use this color for transparency
        let transparent = (255, 0, 255).into();

        let ocm_surface = &ocm_table.surface();

        for sprite in self.data.iter().flatten() {
            let sprite_rect = ocm_table.obj_rectangle(sprite);
            let src_iter =
                RectangleIterator::new_with_rectangle(ocm_surface.dimensions(), sprite_rect);

            let (x, y) = sprite.position();
            let dest_rect = Rectangle2D::new((x as _, y as _).into(), sprite_rect.dimensions);
            let dest_iter = RectangleIterator::new_with_rectangle(surface.dimensions(), dest_rect);

            src_iter.zip(dest_iter).for_each(|(src_pos, dest_pos)| {
                let src_value = ocm_surface.get_value(src_pos);
                if src_value == transparent {
                    return;
                }

                // TODO: Perform conversion with dedicated structs or something.
                //       Think about lossy vs non-lossy and how that should be reflected in the code.
                let dest_value = Rgba8888 {
                    r: src_value.r,
                    g: src_value.g,
                    b: src_value.b,
                    a: 255,
                };

                surface.set_value(dest_pos, &dest_value);
            });
        }
    }
}
