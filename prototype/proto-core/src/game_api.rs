//! This module contains the API from the game. It is intended to be used by the core implementation
//! for interaction with the game.

use std::ops::Deref;


/// ROM data.
///
/// This is usually a custom section in the WASM binary and contains assets for the game that are to
/// be used by the core, such as graphics and sound data. Such assets are normally not mutable or
/// generated at run-time and as such do not need to cross the WASM ABI. A game implementation can
/// pass references to parts of the ROM data to the core (essentially an offset and a size). Such a
/// reference is called a [RomDataRecord].
pub struct RomData {
    data: Vec<u8>,
}

impl RomData {
    pub fn new(data: Vec<u8>) -> Self {
        RomData { data }
    }

    pub fn record(&self, ptr: u32, size: u32) -> RomDataRecord {
        let start = ptr as usize;
        let end = start + size as usize;
        RomDataRecord::new(start, end)
    }
}

impl Deref for RomData {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl AsRef<[u8]> for RomData {
    fn as_ref(&self) -> &[u8] {
        self.deref()
    }
}

/// A record inside a [RomData].
///
/// Essentially, a record consists of an offset (or pointer) and a size.
pub struct RomDataRecord {
    start: usize,
    end: usize,
}

impl RomDataRecord {
    fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn slice<'rom>(&self, rom_data: &'rom RomData) -> &'rom [u8] {
        &rom_data[self.start..self.end]
    }
}