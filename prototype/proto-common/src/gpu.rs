use paste::paste;

/// The position of an entity in the scene.
///
/// This is almost the same as the screen position, except that an entity can be (partially)
/// off-screen, but still be considered part of the scene. This helps creating a virtual space for
/// things like sprites, where such behavior is common. It is up to the rendering code to figure out
/// whether an object should be drawn and where to cut off the drawing, if necessary.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ScenePosition {
    pub x: i16,
    pub y: i16,
}

impl ScenePosition {
    pub fn new(x: i16, y: i16) -> Self {
        Self { x, y }
    }
}

impl From<(i16, i16)> for ScenePosition {
    fn from(tuple: (i16, i16)) -> Self {
        Self::new(tuple.0, tuple.1)
    }
}

bit_struct!(OamEntryInternal, u32,
    { pos_x: u8 @ 0; 0xFF }
    { pos_y: u8 @ 8; 0xFF }
    { char_table_index: u8 @ 16 ; 0xFF }
    { palette_table_index: u8 @ 24 ; 0b111 }
    { pos_x_neg: u8 @ 27; 0b1 }
    { pos_y_neg: u8 @ 28; 0b1 }
    { flip_x: u8 @ 29; 0b1 }
    { flip_y: u8 @ 30; 0b1 }
);

/// An entry in the OAM table.
///
/// The entry can be converted to an [u32] and sent from the game to the core.
///
/// The internal format is as follows:
/// * Bits 0-7: X-position.
/// * Bits 8-15: Y-position.
/// * Bits 16-23: Character table index.
/// * Bits 24-26: Palette table index.
/// * Bit 27: X-position negative flag.
/// * Bit 28: Y-position negative flag.
/// * Bit 29: Horizontal flip flag.
/// * Bit 30: Vertical flip flag.
/// * Bit 31: Unused.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct OamEntry(OamEntryInternal);

impl From<u32> for OamEntry {
    fn from(value: u32) -> Self {
        Self(value.into())
    }
}

impl OamEntry {
    /// Retrieves the [ScenePosition].
    pub fn position(&self) -> ScenePosition {
        let mut x = self.0.pos_x() as i16;
        if self.0.pos_x_neg() != 0 {
            x *= -1;
        }

        let mut y = self.0.pos_y() as i16;
        if self.0.pos_y_neg() != 0 {
            y *= -1;
        }

        (x, y).into()
    }

    /// Sets the [ScenePosition].
    pub fn set_position(&mut self, position: ScenePosition) {
        self.0.set_pos_x(position.x.abs() as u8);
        self.0.set_pos_x_neg(position.x.is_negative() as u8);
        self.0.set_pos_y(position.y.abs() as u8);
        self.0.set_pos_y_neg(position.y.is_negative() as u8);
    }

    /// Retrieves the horizontal-flip flag.
    pub fn h_flip(&self) -> bool {
        self.0.flip_x() != 0
    }

    /// Sets the horizontal-flip flag.
    pub fn set_h_flip(&mut self, flip: bool) {
        self.0.set_flip_x(flip as u8);
    }

    /// Retrieves the vertical-flip flag.
    pub fn v_flip(&self) -> bool {
        self.0.flip_y() != 0
    }

    /// Sets the vertical-flip flag.
    pub fn set_v_flip(&mut self, flip: bool) {
        self.0.set_flip_y(flip as u8);
    }

    /// Retrieves the character table index.
    pub fn char_table_index(&self) -> u8 {
        self.0.char_table_index()
    }

    /// Sets the character table index.
    pub fn set_char_table_index(&mut self, index: u8) {
        self.0.set_char_table_index(index)
    }

    /// Retrieves the palette table index.
    pub fn palette_table_index(&self) -> u8 {
        self.0.palette_table_index()
    }

    /// Sets the palette table index.
    pub fn set_palette_table_index(&mut self, index: u8) {
        self.0.set_palette_table_index(index)
    }
}

#[cfg(test)]
mod tests_oam_entry {
    use crate::gpu::OamEntry;

    // pos_x: 0xAC
    // pos_y: 0x13
    // char_table_index: 5
    // palette_table_index: 4
    // pos_x_neg: 0
    // pos_y_neg: 1
    // flip_x: 1
    // flip_y: 0
    const TEST_VAL: u32 = 0x340513AC;

    #[test]
    fn zero() {
        let subject: OamEntry = 0.into();
        assert_eq!(subject.0.value, 0);
        assert_eq!(subject.position(), (0, 0).into());
        assert_eq!(subject.h_flip(), false);
        assert_eq!(subject.v_flip(), false);
    }

    #[test]
    fn getters() {
        let subject: OamEntry = TEST_VAL.into();
        assert_eq!(subject.0.value, TEST_VAL);
        assert_eq!(subject.position(), (0xAC, -0x13).into());
        assert_eq!(subject.h_flip(), true);
        assert_eq!(subject.v_flip(), false);
        assert_eq!(subject.char_table_index(), 5);
        assert_eq!(subject.palette_table_index(), 4);
    }

    #[test]
    fn setters() {
        let mut subject: OamEntry = TEST_VAL.into();

        let position = (-0x11, 0x22).into();
        let h_flip = true;
        let v_flip = true;
        let char_table_index = 12;
        let palette_table_index = 1;

        subject.set_position(position);
        subject.set_h_flip(h_flip);
        subject.set_v_flip(v_flip);
        subject.set_char_table_index(char_table_index);
        subject.set_palette_table_index(palette_table_index);

        assert_eq!(subject.position(), position);
        assert_eq!(subject.h_flip(), h_flip);
        assert_eq!(subject.v_flip(), v_flip);
        assert_eq!(subject.char_table_index(), char_table_index);
        assert_eq!(subject.palette_table_index(), palette_table_index);
    }
}
