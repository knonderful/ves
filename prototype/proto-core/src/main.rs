use std::path::Path;
use wasmtime::{Store, Linker, Module};
use anyhow::Result;
use std::fmt::{Display, Formatter};
use std::rc::Rc;

pub struct RomDataRecord {
    start: usize,
    end: usize,
}

impl RomDataRecord {
    pub fn from_abi(ptr: u32, size: u32) -> Self {
        let start = ptr as usize;
        let end = start + size as usize;
        RomDataRecord { start, end }
    }

    pub fn slice<'a>(&self, in_slice: &'a [u8]) -> &'a [u8] {
        &in_slice[self.start..self.end]
    }
}

impl Display for RomDataRecord {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("[${:04x}-${:04x}]", self.start, self.end - 1).as_str())?;
        Ok(())
    }
}

fn main() -> Result<()> {
    let wasm_file = std::fs::canonicalize(Path::new("../proto-game/target/wasm32-unknown-unknown/release/proto_game.wasm"))?;
    let rom_data = get_rom_data(&wasm_file)?;

    let store = Store::default();
    let module = Module::from_file(store.engine(), &wasm_file)?;

    // We need the RC here, because we're going to pass this thing into the functions below
    let rom_data = Rc::new(rom_data);

    let mut linker = Linker::new(&store);

    linker.func("gpu", "set_object", move |index: u32, ptr: u32, size: u32| {
        let record = RomDataRecord::from_abi(ptr, size);
        println!("Request to load record {} into object index {}.", record, index);

        std::fs::write(format!("/tmp/object_{}.png", index).as_str(), record.slice(rom_data.as_slice())).unwrap();
    })?;

    let instance = linker.instantiate(&module)?;

    let hello = instance
        .get_func("greet")
        .ok_or(anyhow::format_err!("failed to find `greet` function export"))?
        .get0::<()>()?;

    hello()?;

    Ok(())
}

fn get_rom_data(path: impl AsRef<Path>) -> Result<Vec<u8>> {
    const ROM_DATA: &str = "rom_data";

    let module = parity_wasm::deserialize_file(&path)?;
    let payload = module
        .custom_sections()
        .find(|sect| sect.name() == ROM_DATA)
        .ok_or(anyhow::Error::msg(format!("Could not find rom data (custom section '{}') in {}.", ROM_DATA, path.as_ref().display())))?
        .payload();
    Ok(Vec::from(payload))
}