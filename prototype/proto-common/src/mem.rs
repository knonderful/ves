bit_struct!(
    /// A contiguous block of memory in the game ROM.
    ///
    /// The internal format is as follows:
    /// * Bits 0-31: The offset.
    /// * Bits 32-63: The length.
    #[allow(clippy::len_without_is_empty)]
    pub struct RomBlock {
        value: u64
    }

    impl {
        #[bit_struct_field(shift = 0, mask = 0xFFFFFFFF)]
        /// The offset to the first byte of the block inside the ROM.
        pub fn offset(&self) -> u32;

        #[bit_struct_field(shift = 32, mask = 0xFFFFFFFF)]
        /// The length of the block.
        pub fn len(&self) -> u32;
    }
);
