//! This module contains the API from the game. It is intended to be used by the core implementation
//! for interaction with the game.

use std::ops::Deref;
use std::fmt::{Display, Formatter};

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
        RomDataRecord::new(self, start, end)
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
pub struct RomDataRecord<'rom> {
    rom: &'rom RomData,
    start: usize,
    end: usize,
}

impl<'rom> RomDataRecord<'rom> {
    fn new(rom: &'rom RomData, start: usize, end: usize) -> Self {
        Self { rom, start, end }
    }
}

impl Display for RomDataRecord<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("[${:04x}-${:04x}]", self.start, self.end - 1).as_str())?;
        Ok(())
    }
}

impl<'rom> Deref for RomDataRecord<'rom> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.rom[self.start..self.end]
    }
}

impl AsRef<[u8]> for RomDataRecord<'_> {
    fn as_ref(&self) -> &[u8] {
        self.deref()
    }
}