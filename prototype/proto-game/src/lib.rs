extern crate wee_alloc;

#[macro_use]
mod rom_data;

use wasm_bindgen::prelude::*;
use rom_data::*;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[allow(dead_code)]
#[link_section = "rom_data"]
static ROM_DATA: RomData = insert_rom_data!();

#[wasm_bindgen]
extern {
    pub type Logger;

    #[wasm_bindgen(method)]
    fn info(this: &Logger, msg: &str);

    #[wasm_bindgen]
    fn another(obj: RomDataPointer);
}

type RomDataPointer = *const u8;

#[inline(always)]
fn from_rom<F, T>(func: F) -> RomDataPointer
    where F: FnOnce(&RomData) -> &T
{
    // TODO: Verify that the pointer does not exceed the RomData struct size
    func(rom_data()) as *const T as RomDataPointer
}

#[inline(always)]
fn rom_data() -> &'static RomData
{
    unsafe {
        let null = std::ptr::null();
        &(*null)
    }
}

#[wasm_bindgen]
pub fn greet(_logger: &Logger) {
    another(from_rom(|rom| &rom.version));
    another(from_rom(|rom| &rom.gfx.mario));
    another(from_rom(|rom| &rom.gfx.mario2));
}
