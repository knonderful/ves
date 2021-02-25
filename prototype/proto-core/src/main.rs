mod game_api;

use std::path::Path;
use wasmtime::{Store, Linker, Module};
use anyhow::Result;
use std::rc::Rc;

use game_api::RomData;

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