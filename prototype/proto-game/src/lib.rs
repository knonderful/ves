extern crate wee_alloc;

mod core_api;
mod rom_data;

use rom_data::*;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[allow(dead_code)]
#[link_section = "rom_data"]
static ROM_DATA: RomData = RomData::create();

#[no_mangle]
pub fn greet() {
    let gfx = core_api::gfx();
    gfx.set_object(0, ROM_DATA.gfx().example());
}
