use art_extractor_core::geom::{ArtworkSpaceUnit, Point, Rect, Size};
use art_extractor_core::sprite::{Color, Index, IndexedSurface, Palette, PaletteIndex};
use art_extractor_core::surface::Surface;

/// A data import error.
#[derive(Clone, Debug)]
pub enum DataImportError {
    /// Invalid input data. The provided string contains a more detailed description of the problem.
    InvalidData(String),
}

/// A trait for constructing objects from (raw) SNES data.
///
/// Generally the raw data for the SNES is little-endian.
pub trait FromSnesData<T> where Self: Sized {
    /// Creates an instance from the provided buffer.
    ///
    /// # Parameters
    /// * `data`: A buffer containing the source data in SNES interleaved CHR format (4bpp).
    ///
    /// # Panics
    /// This function panics if the provided buffer is not of the correct size (2 KiB).
    fn from_snes_data(data: T) -> Result<Self, DataImportError>;
}

/// Make a color component from a 5-bit color value.
///
/// # Parameters
/// * A byte with the color data. Only the least-significant 5 bits are considered.
#[inline(always)]
fn make_color_component_5bit(bits: u8) -> u8 {
    // NOTE: "repeat" the bit pattern across the 8 bits to get the most accurate color
    bits << 3 | (bits >> 2) & 0b00000111
}

/// Implementation of [`FromSnesData`] for [`Color`].
///
/// The input data is a tuple where the first byte is the lower byte and the second is the higher byte of the color data. Refer to section
/// A-17 in the SNES developer manual for more information.
impl FromSnesData<(u8, u8)> for Color {
    fn from_snes_data(data: (u8, u8)) -> Result<Self, DataImportError> {
        let (low, high) = data;
        let r = make_color_component_5bit(low);
        let g = make_color_component_5bit(high << 3 | low >> 5);
        let b = make_color_component_5bit(high >> 2);

        Ok(Self::new(r, g, b))
    }
}

#[cfg(test)]
mod test_color {
    use super::{Color, FromSnesData};

    #[test]
    fn test_from_snes_data() {
        let color = Color::from_snes_data((0b11011010, 0b00100100)).unwrap();
        assert_eq!(Color::new(0b11010110, 0b00110001, 0b01001010), color);
        // A negative test for equality on Color to avoid false positives from the previous statement
        assert_ne!(Color::new(0b11010111, 0b00110001, 0b01001010), color);

        let color = Color::from_snes_data((0b10011111, 0b01001011)).unwrap();
        assert_eq!(Color::new(0b11111111, 0b11100111, 0b10010100), color);
        // A negative test for equality on Color to avoid false positives from the previous statement
        assert_ne!(Color::new(0b11111111, 0b11101111, 0b10010100), color);
    }
}

/// The number of bytes for a color in SNES data.
const BYTES_PER_COLOR: usize = 2;
/// The number of colors in an OBJ palette.
const OBJ_PALETTE_NR_COLORS: usize = 16;
/// The number of bytes in an OBJ palette (input SNES data).
const OBJ_PALETTE_SIZE: usize = BYTES_PER_COLOR * OBJ_PALETTE_NR_COLORS;

/// Implementation of [`FromSnesData`] for [`Palette<Color>`].
///
/// The input data is a slice of color entries. Each entry takes 2 bytes. Refer to section A-17 in the SNES developer manual for more
/// information.
impl FromSnesData<&[u8]> for Palette<Color> {
    fn from_snes_data(data: &[u8]) -> Result<Self, DataImportError> {
        if data.len() != OBJ_PALETTE_SIZE {
            return Err(DataImportError::InvalidData(format!("Invalid data length. Expected {} but got {}.", OBJ_PALETTE_SIZE, data.len())));
        }

        let mut palette = Palette::new_filled(OBJ_PALETTE_NR_COLORS, Color::new(0, 0, 0));
        let mut data_iter = data.iter();
        for (_, color) in palette.iter_mut() {
            // The unwraps are OK here because we checked the size of the slice at the beginning of the function
            let low = data_iter.next().unwrap();
            let high = data_iter.next().unwrap();
            *color = Color::from_snes_data((*low, *high))?;
        }

        Ok(palette)
    }
}

#[cfg(test)]
mod test_palette {
    use art_extractor_core::sprite::{Color, Palette};
    use crate::extract::FromSnesData;

    #[test]
    fn test_from_snes_data() {
        const INPUT: [u8; 32] = [0, 0, 159, 75, 28, 59, 179, 37, 0, 0, 159, 75, 223, 99, 255, 115, 0, 0, 255, 127, 255, 127, 255, 127, 27, 115, 255, 127, 255, 127, 255, 127];
        let palette = Palette::from_snes_data(&INPUT).unwrap();

        for (offset, color) in palette.iter().map(|(i, c)| (i.as_usize() * 2, c)) {
            let expected = Color::from_snes_data((INPUT[offset], INPUT[offset + 1])).unwrap();
            assert_eq!(&expected, color);
        }
    }
}

const OBJ_PALETTE_COUNT: usize = 8;

pub struct ObjPalettes {
    palettes: Vec<Palette<Color>>,
}

impl ObjPalettes {
    fn new(palettes: Vec<Palette<Color>>) -> Self {
        Self { palettes }
    }

    pub fn palettes(&self) -> &[Palette<Color>] {
        &self.palettes[..]
    }
}

impl FromSnesData<&[u8]> for ObjPalettes {
    fn from_snes_data(data: &[u8]) -> Result<Self, DataImportError> {
        const EXPECTED_DATA_LEN: usize = OBJ_PALETTE_SIZE * OBJ_PALETTE_COUNT;
        if data.len() != EXPECTED_DATA_LEN {
            return Err(DataImportError::InvalidData(format!("Invalid data length. Expected {} but got {}.", EXPECTED_DATA_LEN, data.len())));
        }

        let mut palettes: Vec<Palette<Color>> = Vec::with_capacity(OBJ_PALETTE_COUNT);
        for input in data.chunks(OBJ_PALETTE_SIZE) {
            palettes.push(Palette::from_snes_data(input)?);
        }

        Ok(ObjPalettes::new(palettes))
    }
}

#[cfg(test)]
mod test_obj_palettes {
    #[test]
    fn fail() {
        // TODO: Parse a bunch of palettes... how do we verify?
        // assert_eq!(1, 2);
    }
}

/// An `OBJ NAME` table. There are two in the scope of the SNES: `OBJ NAME BASE` and `OBJ NAME SELECT`.
pub struct ObjNameTable {
    surface: IndexedSurface,
}

impl ObjNameTable {
    /// The number of 8x8 tiles on the X-axis.
    const TILES_X: ArtworkSpaceUnit = 0x10;
    /// The number of 8x8 tiles on the Y-axis.
    const TILES_Y: ArtworkSpaceUnit = 0x10;
    /// The width of a tile in pixels.
    const TILE_WIDTH: ArtworkSpaceUnit = 8;
    /// The height of a tile in pixels.
    const TILE_HEIGHT: ArtworkSpaceUnit = 8;

    fn read_interleaved_chr(data: &[u8]) -> Result<IndexedSurface, DataImportError> {
        const EXPECTED_LEN: usize = 0x2000;
        if data.len() != EXPECTED_LEN {
            return Err(DataImportError::InvalidData(format!("Expected data length {}, but found {}", EXPECTED_LEN, data.len())));
        }

        let mut surface = IndexedSurface::new(Size::new(Self::TILES_X * Self::TILE_WIDTH, Self::TILES_Y * Self::TILE_HEIGHT));

        let mut data_iter = data.iter();
        let view_size = Size::new(Self::TILE_WIDTH, Self::TILE_HEIGHT);
        // Vertical tile iteration
        for y in 0..Self::TILES_Y {
            // Horizontal tile iteration
            for x in 0..Self::TILES_X {
                // Get a view of the current tile into the surface
                let view = surface.view(Rect::new(Point::new(x * Self::TILE_WIDTH, y * Self::TILE_HEIGHT), view_size));

                // We have to read 2 planes at a time and we have 4 planes in total (4bpp), so we need 2 iterations
                for plane_pair in 0..2 {
                    for row in view.row_iter() {
                        // Read the 2 planes for this row
                        let plane1 = *data_iter.next().unwrap();
                        let plane2 = *data_iter.next().unwrap();

                        let surface_row_data = surface.row_data_mut(&row);
                        // println!("ROW WIDTH: {}", surface_row_data.len()); THE ROW IS NOT THE RIGHT WIDTH... WE NEED TO FIX THE RECT SHIT
                        Self::apply_planes_to_row(surface_row_data, plane_pair * 2, plane1, plane2)
                    }
                }
            }
        }

        // We should have read all data by now. Anything else is a programming error.
        assert!(data_iter.next().is_none());

        Ok(surface)
    }

    /// Applies row data from the SNES interleaved CHR format to the provided buffer.
    ///
    /// # Parameters
    /// * `target_row_data`: The target buffer.
    /// * `bit_offset`: The bit-offset at which to apply the data inside the `PaletteIndex` values.
    /// * `plane1`: The byte containing the bit values for the least-significant value of the row.
    /// * `plane2`: The byte containing the bit values for the most-significant value of the row.
    fn apply_planes_to_row(target_row_data: &mut [PaletteIndex], bit_offset: usize, mut plane1: u8, mut plane2: u8) {
        // Iterate from right to left, since the right-most pixel is the lsb of the source byte
        for pixel in target_row_data.iter_mut().rev() {
            // Apply the two planes to the current pixel
            let mask = (Index::from((plane2 & 0x1) << 1) | Index::from(plane1 & 0x1)) << bit_offset;
            pixel.set_value(pixel.value() | mask);
            // Move to the next bit in the source bytes
            plane1 >>= 1;
            plane2 >>= 1;
        }
    }
}

impl FromSnesData<&[u8]> for ObjNameTable {
    fn from_snes_data(data: &[u8]) -> Result<Self, DataImportError> {
        Ok(Self { surface: Self::read_interleaved_chr(data)? })
    }
}

#[cfg(test)]
mod test_obj_name_table {
    use art_extractor_core::geom::{Point, Rect, Size};
    use art_extractor_core::sprite::{Color, IndexedSurface, Palette, PaletteIndex};
    use art_extractor_core::surface::Surface;
    use bmp::Pixel;
    use crate::extract::ObjPalettes;
    use crate::mesen::Frame;
    use super::{FromSnesData, ObjNameTable};

    #[test]
    fn test_apply_planes_to_row() {
        let plane1 = 0b11010010u8; // the
        let plane2 = 0b00010111u8;
        let plane3 = 0b10111011u8;
        let plane4 = 0b00011010u8;

        let expected: [PaletteIndex; 8] = [
            0b00000101u8, // most-significant bits of plane1-4
            0b00000001u8,
            0b00000100u8,
            0b00001111u8,
            0b00001100u8,
            0b00000010u8,
            0b00001111u8,
            0b00000110u8, // least-significant bits of plane1-4
        ].map(PaletteIndex::from);

        let mut actual = [PaletteIndex::new(0); 8];
        ObjNameTable::apply_planes_to_row(&mut actual, 0, plane1, plane2);
        ObjNameTable::apply_planes_to_row(&mut actual, 2, plane3, plane4);

        assert_eq!(&expected, &actual);
    }

    fn create_bitmap(surface: &IndexedSurface, palette: &Palette<Color>) -> bmp::Image {
        let mut img = bmp::Image::new(surface.size().width, surface.size().height);
        let view = surface.view(Rect::new(Point::new(0, 0), Size::new(surface.size().width - 1, surface.size().height - 1)));
        for (y, row) in view.row_iter().enumerate() {
            for (x, pixel) in surface.row_data(&row).iter().enumerate() {
                let color = palette.get(*pixel).unwrap();
                img.set_pixel(x.try_into().unwrap(), y.try_into().unwrap(),
                              Pixel::new(color.r, color.g, color.b));
            }
        }
        img
    }

    #[test]
    fn test_from_snes_data() {
        let mut json_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        json_path.push("resources/test/frame_189993.json");

        let file = std::fs::File::open(json_path.as_path()).unwrap();
        let frame: Frame = serde_json::from_reader(file).unwrap();

        let obj_name_table = ObjNameTable::from_snes_data(frame.obj_name_base_table.as_slice()).unwrap();
        let palettes = ObjPalettes::from_snes_data(&frame.cgram.as_slice()[0x100..]).unwrap();

        let actual = create_bitmap(&obj_name_table.surface, &palettes.palettes()[5]);
        // actual.save(format!("{}/out.bmp", env!("CARGO_MANIFEST_DIR"))).unwrap();

        let mut expected_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        expected_path.push("resources/test/expected_obj_table.bmp");
        let expected = bmp::open(expected_path).unwrap();

        assert_eq!(expected, actual);
    }
}