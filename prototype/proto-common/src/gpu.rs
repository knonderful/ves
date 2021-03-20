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

bit_struct!(
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
    #[derive(Copy, Clone, Eq, PartialEq)]
    struct OamEntry {
        value: u32
    }

    impl {
        #[bit_struct_field(shift = 0, mask = 0xFF)]
        fn pos_x(&self) -> u8;

        #[bit_struct_field(shift = 8, mask = 0xFF)]
        fn pos_y(&self) -> u8;

        #[bit_struct_field(shift = 16, mask = 0xFF)]
        fn char_table_index_u8(&self) -> u8;

        #[bit_struct_field(shift = 24, mask = 0b111)]
        fn palette_table_index_u8(&self) -> u8;

        #[bit_struct_field(shift = 27, mask = 0b1)]
        fn pos_x_neg(&self) -> u8;

        #[bit_struct_field(shift = 28, mask = 0b1)]
        fn pos_y_neg(&self) -> u8;

        #[bit_struct_field(shift = 29, mask = 0b1)]
        fn flip_x(&self) -> u8;

        #[bit_struct_field(shift = 30, mask = 0b1)]
        fn flip_y(&self) -> u8;
    }
);

#[allow(dead_code)]
impl OamEntry {
    /// Retrieves the [ScenePosition].
    pub fn position(&self) -> ScenePosition {
        let mut x = self.pos_x() as i16;
        if self.pos_x_neg() != 0 {
            x *= -1;
        }

        let mut y = self.pos_y() as i16;
        if self.pos_y_neg() != 0 {
            y *= -1;
        }

        (x, y).into()
    }

    /// Sets the [ScenePosition].
    pub fn set_position(&mut self, position: ScenePosition) {
        self.set_pos_x(position.x.abs() as u8);
        self.set_pos_x_neg(position.x.is_negative() as u8);
        self.set_pos_y(position.y.abs() as u8);
        self.set_pos_y_neg(position.y.is_negative() as u8);
    }

    /// Retrieves the horizontal-flip flag.
    pub fn h_flip(&self) -> bool {
        self.flip_x() != 0
    }

    /// Sets the horizontal-flip flag.
    pub fn set_h_flip(&mut self, flip: bool) {
        self.set_flip_x(flip as u8);
    }

    /// Retrieves the vertical-flip flag.
    pub fn v_flip(&self) -> bool {
        self.flip_y() != 0
    }

    /// Sets the vertical-flip flag.
    pub fn set_v_flip(&mut self, flip: bool) {
        self.set_flip_y(flip as u8);
    }

    /// Retrieves the character table index.
    pub fn char_table_index(&self) -> ObjectCharacterTableEntry {
        self.char_table_index_u8().into()
    }

    /// Sets the character table index.
    pub fn set_char_table_index(&mut self, index: ObjectCharacterTableEntry) {
        self.set_char_table_index_u8(index.into())
    }

    /// Retrieves the palette table index.
    pub fn palette_table_index(&self) -> u8 {
        self.palette_table_index_u8()
    }

    /// Sets the palette table index.
    pub fn set_palette_table_index(&mut self, index: u8) {
        self.set_palette_table_index_u8(index)
    }
}

#[cfg(test)]
mod tests_oam_entry {
    use super::OamEntry;

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
        assert_eq!(subject.value, 0);
        assert_eq!(subject.position(), (0, 0).into());
        assert_eq!(subject.h_flip(), false);
        assert_eq!(subject.v_flip(), false);
        assert_eq!(subject.char_table_index(), 0.into());
        assert_eq!(subject.palette_table_index(), 0);
    }

    #[test]
    fn getters() {
        let subject: OamEntry = TEST_VAL.into();
        assert_eq!(subject.value, TEST_VAL);
        assert_eq!(subject.position(), (0xAC, -0x13).into());
        assert_eq!(subject.h_flip(), true);
        assert_eq!(subject.v_flip(), false);
        assert_eq!(subject.char_table_index(), 5.into());
        assert_eq!(subject.palette_table_index(), 4);
    }

    #[test]
    fn constructor() {
        let subject = OamEntry::new(0xAC, 0x13, 5, 4, 0, 1, 1, 0);
        assert_eq!(subject.value, TEST_VAL);
    }

    #[test]
    fn setters() {
        let mut subject: OamEntry = TEST_VAL.into();

        let position = (-0x11, 0x22).into();
        let h_flip = true;
        let v_flip = true;
        let char_table_index = 12.into();
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

    #[test]
    fn debug() {
        let subject: OamEntry = TEST_VAL.into();
        assert_eq!(
            format!("{:?}", subject).as_str(),
            "OamEntry { pos_x: 172, pos_y: 19, char_table_index_u8: 5, palette_table_index_u8: 4, pos_x_neg: 0, pos_y_neg: 1, flip_x: 1, flip_y: 0 }"
        );
    }
}

bit_struct!(
    /// An entry in the object character table.
    ///
    /// The internal format is as follows:
    /// * Bits 0-3: X-position.
    /// * Bits 4-7: Y-position.
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct ObjectCharacterTableEntry {
        value: u8
    }

    impl {
        #[bit_struct_field(shift = 0, mask = 0xF)]
        /// The X-coordinate in the table.
        pub fn x(&self) -> u8;

        #[bit_struct_field(shift = 4, mask = 0xF)]
        /// The Y-coordinate in the table.
        pub fn y(&self) -> u8;
    }
);

#[cfg(test)]
mod tests_obj_char_table_entry {
    use super::ObjectCharacterTableEntry;

    // x: 0xC
    // y: 0xA
    const TEST_VAL: u8 = 0xAC;

    #[test]
    fn zero() {
        let subject: ObjectCharacterTableEntry = 0.into();
        assert_eq!(subject.value, 0);
        assert_eq!(subject.x(), 0);
        assert_eq!(subject.y(), 0);
    }

    #[test]
    fn getters() {
        let subject: ObjectCharacterTableEntry = TEST_VAL.into();
        assert_eq!(subject.value, TEST_VAL);
        assert_eq!(subject.x(), 0xC);
        assert_eq!(subject.y(), 0xA);
    }

    #[test]
    fn constructor() {
        let subject = ObjectCharacterTableEntry::new(0xC, 0xA);
        assert_eq!(subject.value, TEST_VAL);
    }

    #[test]
    fn setters() {
        let mut subject: ObjectCharacterTableEntry = TEST_VAL.into();

        let x = 0x8;
        let y = 0xA;

        subject.set_x(x);
        subject.set_y(y);

        assert_eq!(subject.x(), x);
        assert_eq!(subject.y(), y);
    }

    #[test]
    fn debug() {
        let subject: ObjectCharacterTableEntry = TEST_VAL.into();
        assert_eq!(
            format!("{:?}", subject).as_str(),
            "ObjectCharacterTableEntry { x: 12, y: 10 }"
        );
    }
}

bit_struct!(
    /// An entry in a palette table. A palette table is always at most 8 entries in size.
    ///
    /// The internal format is as follows:
    /// * Bits 0-3: Index.
    /// * Bits 4-7: Unused.
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct PaletteTableEntry {
        value: u8
    }

    impl {
        #[bit_struct_field(shift = 0, mask = 0b111)]
        /// The index into the palette table.
        pub fn index(&self) -> u8;
    }
);

#[cfg(test)]
mod tests_palette_table_entry {
    use super::PaletteTableEntry;

    // index: 6
    const TEST_VAL: u8 = 6;

    #[test]
    fn zero() {
        let subject: PaletteTableEntry = 0.into();
        assert_eq!(subject.value, 0);
        assert_eq!(subject.index(), 0);
    }

    #[test]
    fn getters() {
        let subject: PaletteTableEntry = TEST_VAL.into();
        assert_eq!(subject.value, TEST_VAL);
        assert_eq!(subject.index(), 6);
    }

    #[test]
    fn constructor() {
        let subject = PaletteTableEntry::new(6);
        assert_eq!(subject.value, TEST_VAL);
    }

    #[test]
    fn setters() {
        let mut subject: PaletteTableEntry = TEST_VAL.into();

        let index = 3;

        subject.set_index(index);

        assert_eq!(subject.index(), index);
    }

    #[test]
    fn debug() {
        let subject: PaletteTableEntry = TEST_VAL.into();
        assert_eq!(
            format!("{:?}", subject).as_str(),
            "PaletteTableEntry { index: 6 }"
        );
    }
}

bit_struct!(
    /// An entry in a palette. Note that not all palettes support the full resolution of 16 entries.
    ///
    /// The internal format is as follows:
    /// * Bits 0-3: Index.
    /// * Bits 4-7: Unused.
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct PaletteEntry {
        value: u8
    }

    impl {
        #[bit_struct_field(shift = 0, mask = 0xF)]
        /// The index.
        pub fn index(&self) -> u8;
    }
);

#[cfg(test)]
mod tests_palette_entry {
    use super::PaletteEntry;

    // index: 6
    const TEST_VAL: u8 = 6;

    #[test]
    fn zero() {
        let subject: PaletteEntry = 0.into();
        assert_eq!(subject.value, 0);
        assert_eq!(subject.index(), 0);
    }

    #[test]
    fn getters() {
        let subject: PaletteEntry = TEST_VAL.into();
        assert_eq!(subject.value, TEST_VAL);
        assert_eq!(subject.index(), 6);
    }

    #[test]
    fn constructor() {
        let subject = PaletteEntry::new(6);
        assert_eq!(subject.value, TEST_VAL);
    }

    #[test]
    fn setters() {
        let mut subject: PaletteEntry = TEST_VAL.into();

        let index = 3;

        subject.set_index(index);

        assert_eq!(subject.index(), index);
    }

    #[test]
    fn debug() {
        let subject: PaletteEntry = TEST_VAL.into();
        assert_eq!(
            format!("{:?}", subject).as_str(),
            "PaletteEntry { index: 6 }"
        );
    }
}

bit_struct!(
    /// A color in a palette.
    ///
    /// The internal format is as follows:
    /// * Bits 0-4: Red component.
    /// * Bits 5-9: Green component.
    /// * Bits 10-14: Blue component.
    /// * Bit 14: Unused.
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct PaletteColor {
        value: u16
    }

    impl {
        #[bit_struct_field(shift = 0, mask = 0b11111)]
        /// The red color component.
        pub fn r(&self) -> u8;
        #[bit_struct_field(shift = 5, mask = 0b11111)]
        /// The green color component.
        pub fn g(&self) -> u8;
        #[bit_struct_field(shift = 10, mask = 0b11111)]
        /// The blue color component.
        pub fn b(&self) -> u8;
    }
);

#[cfg(test)]
mod tests_palette_color {
    use super::PaletteColor;

    // r: 12
    // g: 22
    // b: 7
    //                        b     g     r
    const TEST_VAL: u16 = 0b0_00111_10110_01100;

    #[test]
    fn zero() {
        let subject: PaletteColor = 0.into();
        assert_eq!(subject.value, 0);
        assert_eq!(subject.r(), 0);
        assert_eq!(subject.g(), 0);
        assert_eq!(subject.b(), 0);
    }

    #[test]
    fn getters() {
        let subject: PaletteColor = TEST_VAL.into();
        assert_eq!(subject.value, TEST_VAL);
        assert_eq!(subject.r(), 12);
        assert_eq!(subject.g(), 22);
        assert_eq!(subject.b(), 7);
    }

    #[test]
    fn constructor() {
        let subject = PaletteColor::new(12, 22, 7);
        assert_eq!(subject.value, TEST_VAL);
    }

    #[test]
    fn setters() {
        let mut subject: PaletteColor = TEST_VAL.into();

        let r = 3;
        let g = 15;
        let b = 29;

        subject.set_r(r);
        subject.set_g(g);
        subject.set_b(b);

        assert_eq!(subject.r(), r);
        assert_eq!(subject.g(), g);
        assert_eq!(subject.b(), b);
    }

    #[test]
    fn debug() {
        let subject: PaletteColor = TEST_VAL.into();
        assert_eq!(
            format!("{:?}", subject).as_str(),
            "PaletteColor { r: 12, g: 22, b: 7 }"
        );
    }
}