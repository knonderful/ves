#![allow(dead_code)]

use art_extractor_core::geom::{ArtworkSpaceUnit, Point, Size};
use art_extractor_core::sprite::{Color, Index, Palette, PaletteIndex};
use art_extractor_core::surface::{IntoUsize, Surface};

/// A data import error.
#[derive(Clone, Debug, Eq, PartialEq)]
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

art_extractor_core::sized_surface!(pub ObjNameTableSurface, PaletteIndex, 128, 256, PaletteIndex::new(0));

/// An `OBJ NAME` table. This table contains all the graphics data for objects. In VRAM the data is stored in two separate tables:
/// `OBJ NAME BASE` and `OBJ NAME SELECT`. The SNES treats the concatenation of the two as one table for looking up sprite data. See
/// sections A-1 through A-4 in the SNES Developer Manual for more information.
pub struct ObjNameTable {
    surface: ObjNameTableSurface,
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

    /// Creates an [`IndexedSurface`] from 4bpp interleaved CHR data.
    ///
    /// # Parameters
    /// * `obj_name_base`: A slice of 0x2000 bytes containing the CHR data for `OBJ NAME BASE`.
    /// * `obj_name_select`: A slice of 0x2000 bytes containing the CHR data for `OBJ NAME SELECT`.
    ///
    /// # Panics
    /// If the provided slice is not exactly 0x2000 bytes in size.
    fn read_interleaved_chr(obj_name_base: &[u8], obj_name_select: &[u8]) -> Result<ObjNameTableSurface, DataImportError> {
        const EXPECTED_LEN: usize = 0x2000;
        if obj_name_base.len() != EXPECTED_LEN {
            return Err(DataImportError::InvalidData(format!("Expected OBJ NAME BASE length {}, but found {}", EXPECTED_LEN, obj_name_base.len())));
        }
        if obj_name_select.len() != EXPECTED_LEN {
            return Err(DataImportError::InvalidData(format!("Expected OBJ NAME SELECT length {}, but found {}", EXPECTED_LEN, obj_name_select.len())));
        }

        let mut surface = ObjNameTableSurface::new();

        Self::read_name_table_into_surface(&mut surface, obj_name_base, 0);
        Self::read_name_table_into_surface(&mut surface, obj_name_select, Self::TILES_Y);

        Ok(surface)
    }

    fn read_name_table_into_surface(surface: &mut ObjNameTableSurface, obj_name_data: &[u8], y_offset: ArtworkSpaceUnit) {
        use art_extractor_core::surface::Offset;

        let mut data_iter = obj_name_data.iter();

        // Vertical tile iteration
        for tile_y in 0..Self::TILES_Y {
            // Horizontal tile iteration
            for tile_x in 0..Self::TILES_X {

                // We have to read 2 planes at a time and we have 4 planes in total (4bpp), so we need 2 iterations
                for plane_pair in 0..2 {
                    for pixel_y in 0..Self::TILE_HEIGHT {
                        let plane1 = *data_iter.next().unwrap();
                        let plane2 = *data_iter.next().unwrap();
                        let offset = surface.offset((tile_x * Self::TILE_WIDTH, (y_offset + tile_y) * Self::TILE_HEIGHT + pixel_y).into()).unwrap();
                        let surface_row_data = &mut surface.data_mut()[offset..offset + Self::TILE_WIDTH.into_usize()];
                        Self::apply_planes_to_row(surface_row_data, plane_pair * 2, plane1, plane2)
                    }
                }
            }
        }

        // We should have read all data by now. Anything else is a programming error.
        assert!(data_iter.next().is_none());
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

impl FromSnesData<(&[u8], &[u8])> for ObjNameTable {
    fn from_snes_data(data: (&[u8], &[u8])) -> Result<Self, DataImportError> {
        Ok(Self { surface: Self::read_interleaved_chr(data.0, data.1)? })
    }
}

#[cfg(test)]
mod test_obj_name_table {
    use art_extractor_core::sprite::{Color, Palette, PaletteIndex};
    use art_extractor_core::surface::{Surface, surface_iterate};
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

    fn create_bitmap(surface: &super::ObjNameTableSurface, palette: &Palette<Color>) -> bmp::Image {
        let mut img = bmp::Image::new(surface.size().width, surface.size().height);

        let rect = surface.size().as_rect();
        let mut pos_iter = (0..rect.height())
            .flat_map(|y| {
                std::iter::repeat(y)
                    .zip(0..rect.width())
            })
            .map(|(y, x)| (u32::from(x), u32::from(y)));

        let transparent = Color::new(255, 0, 255);
        surface_iterate(surface.size(), rect, false, false, |index| {
            let pixel = surface.data[index];
            let (x, y) = pos_iter.next().unwrap();
            let color = match pixel.value() {
                0 => &transparent,
                _ => palette.get(pixel).unwrap(),
            };
            img.set_pixel(x, y, Pixel::new(color.r, color.g, color.b));
        }).unwrap();
        img
    }

    #[test]
    fn test_from_snes_data() {
        let mut json_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        json_path.push("resources/test/frame_199250.json");

        let file = std::fs::File::open(json_path.as_path()).unwrap();
        let frame: Frame = serde_json::from_reader(file).unwrap();

        let obj_name_table = ObjNameTable::from_snes_data((frame.obj_name_base_table.as_slice(), frame.obj_name_select_table.as_slice())).unwrap();
        let palettes = ObjPalettes::from_snes_data(&frame.cgram.as_slice()[0x100..]).unwrap();

        let actual = create_bitmap(&obj_name_table.surface, &palettes.palettes()[5]);
        // actual.save(format!("{}/target/out.bmp", env!("CARGO_MANIFEST_DIR"))).unwrap(); // FOR JUST LOOKING
        // actual.save(format!("{}/resources/test/expected_obj_table.bmp", env!("CARGO_MANIFEST_DIR"))).unwrap(); // FOR UPDATING

        let mut expected_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        expected_path.push("resources/test/expected_obj_table.bmp");
        let expected = bmp::open(expected_path).unwrap();

        assert_eq!(expected, actual);
    }
}

/// An OBJ size.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ObjSize {
    /// Small OBJ size: 8x8 pixels.
    Small,
    /// Medium OBJ size: 16x16 pixels.
    Medium,
    /// Large OBJ size: 32x32 pixels.
    Large,
    /// Extra large OBJ size: 64x64 pixels.
    ExtraLarge,
}

impl ObjSize {
    /// Retrieves the [`Size`].
    pub fn size(&self) -> Size {
        let pixel_size = self.pixel_size();
        Size::new(pixel_size, pixel_size)
    }

    fn pixel_size(&self) -> ArtworkSpaceUnit {
        match self {
            ObjSize::Small => 8,
            ObjSize::Medium => 16,
            ObjSize::Large => 32,
            ObjSize::ExtraLarge => 64,
        }
    }
}

#[cfg(test)]
mod test_obj_size {
    use art_extractor_core::geom::Size;
    use super::ObjSize;

    #[test]
    fn test_size() {
        assert_eq!(Size::new(8, 8), ObjSize::Small.size());
        assert_eq!(Size::new(16, 16), ObjSize::Medium.size());
        assert_eq!(Size::new(32, 32), ObjSize::Large.size());
        assert_eq!(Size::new(64, 64), ObjSize::ExtraLarge.size());
    }
}

/// An `OBJ SIZE SELECT`.
///
/// Refer to Chapter 27 of the SNES Developer Manual for more information.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ObjSizeSelect {
    /// Small: [`ObjSize::Small`], Medium: [`ObjSize::Medium`].
    SM,
    /// Small: [`ObjSize::Small`], Medium: [`ObjSize::Large`].
    SL,
    /// Small: [`ObjSize::Small`], Medium: [`ObjSize::ExtraLarge`].
    SXL,
    /// Small: [`ObjSize::Medium`], Medium: [`ObjSize::Large`].
    ML,
    /// Small: [`ObjSize::Medium`], Medium: [`ObjSize::ExtraLarge`].
    MXL,
    /// Small: [`ObjSize::Large`], Medium: [`ObjSize::ExtraLarge`].
    LXL,
}

impl FromSnesData<u8> for ObjSizeSelect {
    fn from_snes_data(data: u8) -> Result<Self, DataImportError> {
        use ObjSizeSelect::*;
        match data {
            0 => Ok(SM),
            1 => Ok(SL),
            2 => Ok(SXL),
            3 => Ok(ML),
            4 => Ok(MXL),
            5 => Ok(LXL),
            _ => Err(DataImportError::InvalidData(format!("Unexpected OBJ SIZE SELECT value: {}.", data)))
        }
    }
}

impl ObjSizeSelect {
    /// Retrieves the "small" variant.
    pub fn small(&self) -> ObjSize {
        use ObjSizeSelect::*;
        match self {
            SM | SL | SXL => ObjSize::Small,
            ML | MXL => ObjSize::Medium,
            LXL => ObjSize::Large,
        }
    }

    /// Retrieves the "large" variant.
    pub fn large(&self) -> ObjSize {
        use ObjSizeSelect::*;
        match self {
            SM => ObjSize::Medium,
            ML | SL => ObjSize::Large,
            SXL | MXL | LXL => ObjSize::ExtraLarge,
        }
    }
}

#[cfg(test)]
mod test_obj_size_select {
    use crate::extract::{DataImportError, FromSnesData, ObjSize, ObjSizeSelect};

    #[test]
    fn test_small() {
        assert_eq!(ObjSizeSelect::SM.small(), ObjSize::Small);
        assert_eq!(ObjSizeSelect::SL.small(), ObjSize::Small);
        assert_eq!(ObjSizeSelect::SXL.small(), ObjSize::Small);
        assert_eq!(ObjSizeSelect::ML.small(), ObjSize::Medium);
        assert_eq!(ObjSizeSelect::MXL.small(), ObjSize::Medium);
        assert_eq!(ObjSizeSelect::LXL.small(), ObjSize::Large);
    }

    #[test]
    fn test_large() {
        assert_eq!(ObjSizeSelect::SM.large(), ObjSize::Medium);
        assert_eq!(ObjSizeSelect::SL.large(), ObjSize::Large);
        assert_eq!(ObjSizeSelect::SXL.large(), ObjSize::ExtraLarge);
        assert_eq!(ObjSizeSelect::ML.large(), ObjSize::Large);
        assert_eq!(ObjSizeSelect::MXL.large(), ObjSize::ExtraLarge);
        assert_eq!(ObjSizeSelect::LXL.large(), ObjSize::ExtraLarge);
    }

    #[test]
    fn test_from_snes_data() {
        assert_eq!(Ok(ObjSizeSelect::SM), ObjSizeSelect::from_snes_data(0));
        assert_eq!(Ok(ObjSizeSelect::SL), ObjSizeSelect::from_snes_data(1));
        assert_eq!(Ok(ObjSizeSelect::SXL), ObjSizeSelect::from_snes_data(2));
        assert_eq!(Ok(ObjSizeSelect::ML), ObjSizeSelect::from_snes_data(3));
        assert_eq!(Ok(ObjSizeSelect::MXL), ObjSizeSelect::from_snes_data(4));
        assert_eq!(Ok(ObjSizeSelect::LXL), ObjSizeSelect::from_snes_data(5));
        for i in 6..=255 {
            assert_eq!(Err(DataImportError::InvalidData(format!("Unexpected OBJ SIZE SELECT value: {}.", i))), ObjSizeSelect::from_snes_data(i));
        }
    }
}

/// An index into [`ObjNameTable`].
///
/// This basically corresponds to the `NAME` part of the `OBJECT DATA` of page A-3 of the SNES Developer Manual.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct ObjNameTableIndex {
    /// A flag that specifies whether this is an entry in the `OBJ NAME BASE` table. If `false`, this is an entry in the `OBJ NAME SELECT`
    /// table.
    is_base: bool,
    /// The index into the specified table.
    index: u8,
}

impl ObjNameTableIndex {
    fn for_base(index: u8) -> Self {
        Self { is_base: true, index }
    }

    fn for_select(index: u8) -> Self {
        Self { is_base: false, index }
    }
}

impl FromSnesData<u16> for ObjNameTableIndex {
    fn from_snes_data(data: u16) -> Result<Self, DataImportError> {
        let is_base = (0x100 & data) == 0;
        let index = (0xFF & data) as u8;
        Ok(Self { is_base, index })
    }
}

/// The `OBJECT DATA` as described on page A-3 of the SNES Developer Manual.
#[derive(Clone, Debug, Eq, PartialEq)]
struct ObjData {
    /// The `NAME` or `CHARACTER CODE NUMBER` field. This is an index into [`ObjNameTable`].
    name: ObjNameTableIndex,
    /// The `COLOR PALETTE SELECT` field. This is the index into [`ObjPalettes`].
    color: u8,
    /// The `H` component of the `H/V FLIP` field. Horizontal flip flag.
    h_flip: bool,
    /// The `V` component of the `H/V FLIP` field. Vertical flip flag.
    v_flip: bool,
    /// The combination of the `OBJ H-POSITION` and `OBJ V-POSITION` fields. Position on the screen.
    position: Point,
    /// The "Size Large/Small" field. The value is `true` if the size is "large", otherwise `false`.
    size_large: bool,
}

impl FromSnesData<(u8, u8, u8, u8, u8)> for ObjData {
    fn from_snes_data((low1, low2, low3, low4, high): (u8, u8, u8, u8, u8)) -> Result<Self, DataImportError> {
        let mut low2 = low2;

        let name = ((low2 & 0b1) as u16) << 8 | (low1 as u16);
        let name = ObjNameTableIndex::from_snes_data(name).unwrap();

        low2 >>= 1;
        let color = low2 & 0b111;
        low2 >>= 5; // NOTE: Skipping OBJ PRIORITY
        let h_flip = low2 & 0b1 != 0;
        let v_flip = low2 & 0b10 != 0;

        let pos_x = ArtworkSpaceUnit::from(low3);
        let pos_y = ArtworkSpaceUnit::from(high & 0b1) << 8 | ArtworkSpaceUnit::from(low4);
        let position = (pos_x, pos_y).into();
        let size_large = high & 0b10 != 0;

        Ok(Self { name, color, h_flip, v_flip, position, size_large })
    }
}

#[cfg(test)]
mod test_obj_data {
    use art_extractor_core::geom::Point;
    use crate::extract::{FromSnesData, ObjData, ObjNameTableIndex};

    #[test]
    fn test_from_snes_data() {
        let obj = ObjData::from_snes_data((0b01011101, 0b10100101, 0b01100101, 0b01101111, 0b11100011)).unwrap();
        assert_eq!(ObjNameTableIndex::for_select(93), obj.name);
        assert_eq!(2, obj.color);
        assert_eq!(false, obj.h_flip);
        assert_eq!(true, obj.v_flip);
        assert_eq!(true, obj.size_large);
        assert_eq!(Point::new(101, 367), obj.position);

        let obj = ObjData::from_snes_data((0b01000101, 0b01111110, 0b01110100, 0b01101000, 0b11000100)).unwrap();
        assert_eq!(ObjNameTableIndex::for_base(69), obj.name);
        assert_eq!(7, obj.color);
        assert_eq!(true, obj.h_flip);
        assert_eq!(false, obj.v_flip);
        assert_eq!(false, obj.size_large);
        assert_eq!(Point::new(116, 104), obj.position);
    }
}

/// The OAM table as described on page A-3 of the SNES Developer Manual.
#[derive(Clone, Debug, Eq, PartialEq)]
struct OamTable {
    /// The objects. There are 128 entries.
    objects: Vec<ObjData>,
}

impl FromSnesData<&[u8]> for OamTable {
    fn from_snes_data(data: &[u8]) -> Result<Self, DataImportError> {
        const EXPECTED_SIZE: usize = 0x220;
        if data.len() != EXPECTED_SIZE {
            return Err(DataImportError::InvalidData(format!("Invalid data length. Expected {} but got {}.", EXPECTED_SIZE, data.len())));
        }

        let mut low_iter = data[0x00..0x200].iter();
        let mut high_iter = data[0x200..0x220].iter();
        let mut high = 0u8;

        let mut objects = Vec::with_capacity(0x80);
        for i in 0..0x80 {
            let low1 = *low_iter.next().unwrap();
            let low2 = *low_iter.next().unwrap();
            let low3 = *low_iter.next().unwrap();
            let low4 = *low_iter.next().unwrap();

            // Every OBJ uses 2 bits in a "high table" byte. That means we need to grab a new byte every 4 OBJs.
            if i % 4 == 0 {
                high = *high_iter.next().unwrap();
            } else {
                high >>= 2;
            }

            objects.push(ObjData::from_snes_data((low1, low2, low3, low4, high))?)
        }

        // We should have read all data by now. Anything else is a programming error.
        assert!(low_iter.next().is_none());
        assert!(high_iter.next().is_none());

        Ok(OamTable { objects })
    }
}

#[cfg(test)]
mod test_oam_table {
    use crate::extract::{FromSnesData, OamTable};
    use crate::mesen::Frame;

    #[test]
    fn test_from_snes_data() {
        let mut json_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        json_path.push("resources/test/frame_199250.json");

        let file = std::fs::File::open(json_path.as_path()).unwrap();
        let frame: Frame = serde_json::from_reader(file).unwrap();

        // Currently we only test that the unwrap doesn't fail, which means we at least read the right amount of data.
        OamTable::from_snes_data(frame.oam.as_slice()).unwrap();
    }
}