use log::info;
use ves_proto_common::gpu::{OamTableEntry, OamTableIndex};
use ves_proto_common::log::LogLevel;
use ves_proto_logger::Logger;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[link(wasm_import_module = "log")]
extern "C" {
    /// Core function for logging.
    ///
    /// This function pointer is intended to be passed into a [`Logger`] instance.
    ///
    /// # Arguments
    ///
    /// * `level`: The [`LogLevel`](ves_proto_common::log::LogLevel).
    /// * `ptr`: A pointer to the start of the message.
    /// * `len`: The length of the message in bytes.
    #[link_name = "log"]
    fn core_log_log(level: u32, ptr: *const u8, len: usize);
}

#[link(wasm_import_module = "gpu")]
extern "C" {
    /// Core function for setting an entry in the OAM table.
    ///
    /// # Arguments
    ///
    /// * `index`: The [`OamTableIndex`](ves_proto_common::gpu::OamTableIndex).
    /// * `entry`: The [`OamTableEntry`](ves_proto_common::gpu::OamTableEntry).
    #[link_name = "oam_set"]
    fn core_gpu_oam_set(index: u8, entry: u64);
}

#[no_mangle]
pub fn create_instance() -> Box<Game> {
    Logger::new(core_log_log).init(Some(LogLevel::Trace)).unwrap();
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
        unsafe {
            let index = OamTableIndex::new(0);
            let entry = OamTableEntry::new(
                10,
                20,
                3,
                1,
                0,
                123,
            );
            core_gpu_oam_set(index.into(), entry.into())
        }
    }
}
