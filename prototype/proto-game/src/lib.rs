extern crate wee_alloc;

mod core_api;
mod rom_data;

use rom_data::*;
use crate::core_api::Core;

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

impl GameInstance {
    fn core(&self) -> &Core {
        &self.core
    }
}

impl Game for GameInstance {
    fn new(core: Core) -> Self {
        GameInstance {
            core,
            frame_count: 0,
        }
    }

    fn step(&mut self) {
        self.frame_count += 1;
        let msg = format!("Frame #{}", &self.frame_count);
        self.core().logger().info(&msg);
        self.core().gpu().objects().set(0, ROM_DATA.gfx().example());
    }
}