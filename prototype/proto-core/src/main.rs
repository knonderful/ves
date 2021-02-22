use std::path::Path;
use wasmtime::{Store, Linker, Module};
use anyhow::Result;
use std::fmt::{Display, Formatter};

pub type RomDataPointer = u32;

pub struct RomDataRecord {
    ptr: RomDataPointer,
    size: u32,
}

impl RomDataRecord {
    pub fn from_abi(ptr: u32, size: u32) -> Self {
        RomDataRecord { ptr, size }
    }
}

impl Display for RomDataRecord {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("[${:04x}-${:04x}]", self.ptr, self.ptr + self.size - 1).as_str())?;
        Ok(())
    }
}

fn main() -> Result<()> {
    let store = Store::default();
    let path = Path::new("../proto-game/target/wasm32-unknown-unknown/release/proto_game.wasm");
    let module = Module::from_file(store.engine(), path)?;

    let mut linker = Linker::new(&store);

    linker.func("gpu", "set_object", |index: u32, ptr: u32, size: u32| {
        let record = RomDataRecord::from_abi(ptr, size);
        println!("Request to load record {} into object index {}.", record, index);
    })?;

    let instance = linker.instantiate(&module)?;

    let hello = instance
        .get_func("greet")
        .ok_or(anyhow::format_err!("failed to find `greet` function export"))?
        .get0::<()>()?;

    hello()?;

    Ok(())
}
