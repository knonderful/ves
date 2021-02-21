extern crate wee_alloc;

mod core_api;
#[macro_use]
mod rom_data;

use wasm_bindgen::prelude::*;
use rom_data::*;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[allow(dead_code)]
#[link_section = "rom_data"]
static ROM_DATA: RomData = RomData::create();

#[wasm_bindgen]
pub fn greet() {
    core_api::call_another(ROM_DATA.gfx().mario());
    core_api::call_another(ROM_DATA.gfx().mario2());
}
