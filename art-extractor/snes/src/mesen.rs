/// A "frame" from a Mesen-S capture session (using `emu_scripts/mesen-s/sprite_extractor.lua`).
///
/// For each game frame the LUA script does the following:
///
/// * Determine the frame number.
/// * Extract the CGRAM (containing the palette data).
/// * Extract the OAM (containing sprite information, sans graphics).
/// * Locate and extract the two object-relevant data tables from VRAM.
///
/// All this gets written into a JSON file (one per frame, as to not run out of memory in the emulator) in the same structure as the `Frame`
/// struct.
#[derive(serde::Deserialize)]
pub struct Frame {
    /// The frame number. This can be useful for autmatically determining animation timings, movement speeds etc.
    pub frame_nr: u64,
    /// The `OBJ SIZE SELECT` from PPU register 0x2100. See Chapter 27 in the SNES Developer Manual.
    pub obj_size_select: u8,
    /// The entire CGRAM table (see page A-17 of book1). This should be 0x200 bytes.
    /// Note that only the latter half of the CGRAM is used for objects (from 0x100), but we copy the entire table to avoid confusion.
    pub cgram: Vec<u8>,
    /// The entire OAM table (see page A-3 of book1). This should be 0x220 bytes.
    pub oam: Vec<u8>,
    /// `OBJ NAME BASE` table from VRAM (see page A-1 and A-2 of book1). This should be 0x2000 bytes.
    pub obj_name_base_table: Vec<u8>,
    /// `OBJ NAME SELECT` table from VRAM (see page A-1 and A-2 of book1). This should be 0x2000 bytes.
    pub obj_name_select_table: Vec<u8>,
}

#[cfg(test)]
mod test_frame {
    use super::Frame;

    /// Tests the JSON deserialization with synthetic input.
    #[test]
    fn test_deserialize_synthetic() {
        const TEST_JSON: &str = r###"{
            "frame_nr": 123,
            "obj_size_select": 2,
            "cgram": [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
            "oam": [10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25],
            "obj_name_base_table": [20, 21, 22, 23, 24, 25, 26, 27, 28, 29],
            "obj_name_select_table": [30, 31, 32, 33, 34, 35, 36, 37, 38, 39]
        }"###;

        let frame: Frame = serde_json::from_str(TEST_JSON).unwrap();
        assert_eq!(frame.frame_nr, 123);
        assert_eq!(frame.obj_size_select, 2);
        assert_eq!(frame.cgram, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
        assert_eq!(frame.oam, vec![10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25]);
        assert_eq!(frame.obj_name_base_table, vec![20, 21, 22, 23, 24, 25, 26, 27, 28, 29]);
        assert_eq!(frame.obj_name_select_table, vec![30, 31, 32, 33, 34, 35, 36, 37, 38, 39]);
    }

    fn hash_value(hashable: &impl std::hash::Hash) -> u64 {
        use std::hash::Hasher;
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        hashable.hash(&mut hasher);
        hasher.finish()
    }

    /// Tests the JSON deserialization with real input. The input file was taken from an actual run of Yoshi's Island in Mesen-S.
    #[test]
    fn test_deserialize_real() {
        let mut file_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        file_path.push("resources/test/frame_199250.json");

        let file = std::fs::File::open(file_path.as_path()).unwrap();
        let frame: Frame = serde_json::from_reader(file).unwrap();
        assert_eq!(frame.frame_nr, 199250);
        assert_eq!(frame.obj_size_select, 0);
        // Not going to verify the content, just the lengths
        assert_eq!(frame.cgram.len(), 0x200);
        assert_eq!(frame.oam.len(), 0x220);
        assert_eq!(frame.obj_name_base_table.len(), 0x2000);
        assert_eq!(frame.obj_name_select_table.len(), 0x2000);
        // A quick and dirty check that depends on internal implementations of slice and DefaultHasher, but it's better than just checking the length
        assert_eq!(hash_value(&frame.obj_name_base_table.as_slice()), 7240137848684959837);
    }
}