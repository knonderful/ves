use crate::gfx::{Unit2D, SliceBackedSurface, Rectangle2D, SliceBackedSurfaceMut, Rgb888};
use proto_common::gpu::OamTableEntry;

/// The width of a character in pixels.
const CHAR_WIDTH: Unit2D = 8;
/// The height of a character in pixels.
const CHAR_HEIGHT: Unit2D = 8;
/// The width of the character table in number of characters.
const OBJ_CHAR_TABLE_WIDTH: Unit2D = 16;
/// The height of the character table in number of characters.
const OBJ_CHAR_TABLE_HEIGHT: Unit2D = 16;
/// The size of the object attribute table in number of entries.
pub const OBJ_ATTR_MEM_SIZE: usize = 32usize;

// TODO: Replace FrameBufferPixel with another pixel type that only stores the NECESSARY data (basically the indices, not the RGBA)
crate::linear_pixel_buffer!(OcmSurfaceBuffer, Rgb888, OBJ_CHAR_TABLE_WIDTH, OBJ_CHAR_TABLE_HEIGHT);

/// A character table.
#[derive(Default)]
pub struct OcmTable {
    surface_buffer: OcmSurfaceBuffer,
}

impl OcmTable {
    pub fn surface(&self) -> SliceBackedSurface<Rgb888> {
        self.surface_buffer.as_surface()
    }

    pub fn surface_mut(&mut self) -> SliceBackedSurfaceMut<Rgb888> {
        self.surface_buffer.as_surface_mut()
    }

    pub fn obj_rectangle(&self, oam_entry: &OamTableEntry) -> Rectangle2D {
        let char_table_index = oam_entry.char_table_index();
        let origin = (char_table_index.x() as Unit2D * CHAR_WIDTH, char_table_index.y() as Unit2D * CHAR_HEIGHT).into();
        // TODO: Support different sized sprites here
        Rectangle2D::new(origin, (CHAR_WIDTH, CHAR_HEIGHT).into())
    }
}