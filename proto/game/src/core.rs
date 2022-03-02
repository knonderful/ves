use log::info;
use ves_proto_common::gpu::{OamTableEntry, OamTableIndex, PaletteColor, PaletteIndex, PaletteTableIndex};
use ves_proto_common::log::LogLevel;
use ves_proto_logger::Logger;

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

    /// Core function for setting an entry in the palette table.
    ///
    /// # Arguments
    ///
    /// * `palette`: The [`PaletteTableIndex`](ves_proto_common::gpu::PaletteTableIndex).
    /// * `index`: The [`PaletteIndex`](ves_proto_common::gpu::PaletteIndex).
    /// * `color`: The [`PaletteColor`](ves_proto_common::gpu::PaletteColor).
    #[link_name = "palette_set"]
    fn core_gpu_palette_set(palette: u8, index: u8, color: u16);
}

pub struct Core;

impl Core {
    pub fn new() -> Self {
        Logger::new(core_log_log)
            .init(Some(LogLevel::Trace))
            .unwrap();
        info!("Logging initialized.");

        Self
    }

    pub fn oam_set(&self, index: &OamTableIndex, entry: &OamTableEntry) {
        unsafe {
            core_gpu_oam_set(index.into(), entry.into());
        }
    }

    pub fn palette_set(&self, palette: &PaletteTableIndex, index: &PaletteIndex, color:&PaletteColor) {
        unsafe {
            core_gpu_palette_set(palette.into(), index.into(), color.into());
        }
    }
}