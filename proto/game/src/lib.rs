use log::info;
use ves_proto_common::log::LogLevel;
use ves_proto_logger::Logger;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[link(wasm_import_module = "log")]
extern "C" {
    #[link_name = "log"]
    fn log_fn(level: u32, ptr: *const u8, len: usize);
}

#[no_mangle]
pub fn create_instance() -> Box<Game> {
    Logger::new(log_fn).init(Some(LogLevel::Trace)).unwrap();
    info!("Logging initialized.");

    Box::new(Game { frame_nr: 1024 })
}

#[no_mangle]
pub fn step(game: &mut Game) {
    game.step();
}

pub struct Game {
    frame_nr: u32,
}

impl Game {
    fn step(&mut self) {
        self.frame_nr += 1;
        info!("At frame {}", self.frame_nr);
    }
}
