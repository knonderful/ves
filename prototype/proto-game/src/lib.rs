extern crate wee_alloc;

mod core_api;
mod rom_data;

use rom_data::*;
use crate::core_api::{Core, ObjectSize, ObjectCharacterTableIndex};
use proto_common::gpu::{OamEntry, OcmTableIndex};

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[allow(dead_code)]
#[link_section = "rom_data"]
static ROM_DATA: RomData = RomData::create();

#[no_mangle]
pub fn create_instance() -> Box<GameInstance> {
    Box::new(GameInstance::new(Core::new()))
}

#[no_mangle]
pub fn step(instance: &mut GameInstance) {
    instance.step();
}

pub trait Game {
    fn new(core: Core) -> Self;
    fn step(&mut self);
}

pub struct GameInstance {
    core: Core,
    frame_count: u64,
}

impl Game for GameInstance {
    fn new(core: Core) -> Self {
        GameInstance {
            core,
            frame_count: 0,
        }
    }

    fn step(&mut self) {
        let gpu = &mut self.core.gpu;
        let objects = &mut gpu.objects;

        if self.frame_count == 0 {
            self.core.logger.info("Initializing.");


            self.core.logger.info("Loading ROM data into object character table.");
            let character_table_index: ObjectCharacterTableIndex = (0, 0).into();
            let char_table = &mut gpu.char_table;
            char_table.load(character_table_index, ROM_DATA.gfx().example().ptr(), ObjectSize::Size8x8);

            let mut entry = OamEntry::default();
            entry.set_char_table_index(OcmTableIndex::new(0, 0));
            let idx = 0.into();
            objects.set(&idx, &entry);
        }

        self.frame_count += 1;
        let msg = format!("Frame #{}", &self.frame_count);
        self.core.logger.info(msg.as_str());
        // self.core.gpu.objects.set(0, ROM_DATA.gfx().example());
    }
}