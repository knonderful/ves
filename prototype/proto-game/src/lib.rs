extern crate wee_alloc;

use proto_common::api::{CoreInterface, CoreInterfaceForGame, GameInterface};
use proto_common::gpu::{OamTableEntry, OcmTableIndex};

use rom_data::*;

mod rom_data;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[allow(dead_code)]
#[link_section = "rom_data"]
static ROM_DATA: RomData = RomData::create();

#[link(wasm_import_module = "logger")]
extern {
    #[link_name = "info"]
    fn logger_info(ptr: *const u8, len: usize);
}

#[link(wasm_import_module = "ocm")]
extern {
    #[link_name = "load"]
    fn ocm_load(index: u8, rom_block: u64);
}

#[link(wasm_import_module = "oam")]
extern {
    #[link_name = "set"]
    fn oam_set(index: u8, oam_entry: u32);
}


#[no_mangle]
pub fn create_instance() -> Box<CoreAndGame> {
    let instance = CoreAndGame {
        core: CoreInterfaceForGame {
            logger_info,
            ocm_load,
            oam_set,
        },
        game: Game::new(),
    };

    Box::new(instance)
}

#[no_mangle]
pub fn step(instance: &mut CoreAndGame) {
    instance.game.step(&mut instance.core);
}

pub struct CoreAndGame {
    core: CoreInterfaceForGame,
    game: Game,
}

pub struct Game {
    frame_count: u64,
}

impl GameInterface for Game {
    fn new() -> Self {
        Self {
            frame_count: 0,
        }
    }

    fn step(&mut self, core: &mut dyn CoreInterface) {
        if self.frame_count == 0 {
            core.log_info("Initializing.");


            core.log_info("Loading ROM data into object character table.");
            let ocm_index = OcmTableIndex::new(0, 0);
            core.ocm_load(ocm_index, ROM_DATA.gfx().example());

            let mut entry = OamTableEntry::default();
            entry.set_char_table_index(ocm_index);
            let idx = 0.into();
            core.oam_set(idx, entry);
        }

        self.frame_count += 1;
        let msg = format!("Frame #{}", &self.frame_count);
        core.log_info(msg.as_str());
    }
}