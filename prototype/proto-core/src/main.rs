use std::path::Path;
use wasmtime::{Store, Linker, Module};
use anyhow::Result;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
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
    fn new(data: Vec<u8>) -> Self {
        RomData { data }
    }

    fn record(&self, ptr: u32, size: u32) -> RomDataRecord {
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

fn main() -> Result<()> {
    let wasm_file = std::fs::canonicalize(Path::new("../proto-game/target/wasm32-unknown-unknown/release/proto_game.wasm"))?;
    let rom_data = get_rom_data(&wasm_file)?;
    std::fs::write("/tmp/rom_data.bin", &rom_data).unwrap();

    let store = Store::default();
    let module = Module::from_file(store.engine(), &wasm_file)?;

    // We need the RC here, because we're going to pass this thing into the functions below
    let rom_data = Rc::new(rom_data);

    let mut linker = Linker::new(&store);

    linker.func("gpu", "set_object", move |index: u32, ptr: u32, size: u32| {
        let record = rom_data.record(ptr, size);
        println!("Request to load record {} into object index {}.", &record, index);

        std::fs::write(format!("/tmp/object_{}.png", index).as_str(), record).unwrap();
    })?;

    let instance = linker.instantiate(&module)?;

    let hello = instance
        .get_func("greet")
        .ok_or(anyhow::format_err!("failed to find `greet` function export"))?
        .get0::<()>()?;

    hello()?;

    Ok(())
}

fn get_rom_data(path: impl AsRef<Path>) -> Result<RomData> {
    const ROM_DATA: &str = "rom_data";

    let module = parity_wasm::deserialize_file(&path)?;
    let payload = module
        .custom_sections()
        .find(|sect| sect.name() == ROM_DATA)
        .ok_or(anyhow::Error::msg(format!("Could not find rom data (custom section '{}') in {}.", ROM_DATA, path.as_ref().display())))?
        .payload();
    Ok(RomData::new(Vec::from(payload)))
}