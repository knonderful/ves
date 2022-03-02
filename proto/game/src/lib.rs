mod core;

use crate::core::Core;
use log::info;
use ves_proto_common::gpu::{
    OamTableEntry, OamTableIndex, PaletteColor, PaletteIndex, PaletteTableIndex,
};

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[no_mangle]
pub fn create_instance() -> Box<Game> {
    let core = Core::new();
    Box::new(Game { core, frame_nr: 0 })
}

#[no_mangle]
pub fn step(game: &mut Game) {
    game.step();
}

pub struct Game {
    core: Core,
    frame_nr: u32,
}

impl Game {
    fn step(&mut self) {
        self.frame_nr += 1;
        info!("At frame {}", self.frame_nr);

        let index = OamTableIndex::new(0);
        let entry = OamTableEntry::new(10, 20, 3, 1, 0, 123);
        self.core.oam_set(&index, &entry);

        let palette = PaletteTableIndex::new(2);
        let index = PaletteIndex::new(14);
        let color = PaletteColor::new(3, 2, 1);
        self.core.palette_set(&palette, &index, &color);
    }
}
