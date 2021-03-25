bit_struct!(
    /// An entry in the OAM table.
    ///
    /// The entry can be converted to an [u32] and sent from the game to the core.
    ///
    /// The internal format is as follows:
    /// * Bits 0-8: X-position.
    /// * Bits 9-17: Y-position.
    /// * Bits 18-25: Character table index.
    /// * Bits 26-28: Palette table index.
    /// * Bit 29: Horizontal flip flag.
    /// * Bit 30: Vertical flip flag.
    /// * Bit 31: Unused.
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct OamEntry {
        value: u32
    }

    impl {
        #[bit_struct_field(shift = 0, mask = 0x1FF)]
        fn pos_x(&self) -> u16;

        #[bit_struct_field(shift = 9, mask = 0x1FF)]
        fn pos_y(&self) -> u16;

        #[bit_struct_field(shift = 18, mask = 0xFF)]
        fn char_table_index_u8(&self) -> u8;

        #[bit_struct_field(shift = 26, mask = 0b111)]
        fn palette_table_index_u8(&self) -> u8;

        #[bit_struct_field(shift = 29, mask = 0b1)]
        fn flip_x(&self) -> u8;

        #[bit_struct_field(shift = 30, mask = 0b1)]
        fn flip_y(&self) -> u8;
    }

    padding {
        #[bit_struct_field(shift = 31, mask = 0b1)]
        fn unused(&self) -> u8;
    }
);

impl OamEntry {
    /// Retrieves the position of the top-left pixel.
    ///
    /// Note that only the 9 least-significant bits of the coordinates are used.
    pub fn position(&self) -> (u16, u16) {
        (self.pos_x(), self.pos_y())
    }

    /// Sets the position of the top-left pixel.
    ///
    /// Note that only the 9 least-significant bits of the coordinates are used.
    pub fn set_position(&mut self, x: u16, y: u16) {
        self.set_pos_x(x);
        self.set_pos_y(y);
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

    // pos_x: 0x1AC
    // pos_y: 0x13
    // char_table_index: 5
    // palette_table_index: 4
    // flip_x: 1
    // flip_y: 0
    //                        y x pal chr_idx  pos_y     pos_x
    const TEST_VAL: u32 = 0b0_0_1_100_00000101_000010011_110101100;

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
        assert_eq!(subject.position(), (0x1AC, 0x13).into());
        assert_eq!(subject.h_flip(), true);
        assert_eq!(subject.v_flip(), false);
        assert_eq!(subject.char_table_index(), 5.into());
        assert_eq!(subject.palette_table_index(), 4);
    }

    #[test]
    fn constructor() {
        let subject = OamEntry::new(0x1AC, 0x13, 5, 4, 1, 0);
        assert_eq!(subject.value, TEST_VAL);
    }

    #[test]
    fn setters() {
        let mut subject: OamEntry = TEST_VAL.into();

        let position = (0x11, 0x22);
        let h_flip = true;
        let v_flip = true;
        let char_table_index = 12.into();
        let palette_table_index = 1;

        subject.set_position(position.0, position.1);
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
            "OamEntry { pos_x: 428, pos_y: 19, char_table_index_u8: 5, palette_table_index_u8: 4, flip_x: 1, flip_y: 0 }"
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

    padding {
        #[bit_struct_field(shift = 3, mask = 0b11111)]
        fn unused(&self) -> u8;
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

    padding {
        #[bit_struct_field(shift = 4, mask = 0xF)]
        fn unused(&self) -> u8;
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

    padding {
        #[bit_struct_field(shift = 15, mask = 0b1)]
        fn unused(&self) -> u8;
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