use crate::gpu::{OamTableIndex, OamTableEntry, OcmTableIndex};
use crate::mem::RomBlock;

/// The interface that must be implemented by a core.
pub trait CoreInterface {
    /// Sends an info message to the core.
    fn log_info(&self, message: &str);

    /// Loads graphics data from the game ROM into the object character memory.
    ///
    /// # Parameters
    /// * `index`: The [`OcmTableIndex`].
    /// * `rom_block`: The [`RomBlock`].
    fn ocm_load(&mut self, index: OcmTableIndex, rom_block: RomBlock);

    /// Sets an OAM entry.
    ///
    /// # Parameters
    /// * `index`: The [`OamTableIndex`].
    /// * `oam_entry`: The [`OamTableEntry`].
    fn oam_set(&mut self, index: OamTableIndex, entry: OamTableEntry);
}

/// A [`CoreInterface`] for the game side.
pub struct CoreInterfaceForGame {
    /// Function pointer for `logger.info()`.
    pub logger_info: unsafe extern "C" fn(*const u8, usize),

    /// Function pointer for `ocm.load()`.
    pub ocm_load: unsafe extern "C" fn(u8, u64),

    /// Function pointer for `oam.set()`.
    pub oam_set: unsafe extern "C" fn(u8, u32),
}

impl CoreInterface for CoreInterfaceForGame {
    fn log_info(&self, message: &str) {
        unsafe {
            (self.logger_info)(message.as_ptr(), message.len());
        }
    }

    fn ocm_load(&mut self, index: OcmTableIndex, rom_block: RomBlock) {
        unsafe {
            (self.ocm_load)(index.into(), rom_block.into());
        }
    }

    fn oam_set(&mut self, index: OamTableIndex, entry: OamTableEntry) {
        unsafe {
            (self.oam_set)(index.into(), entry.into());
        }
    }
}

/// The interface that must be implemented by a game.
pub trait GameInterface {
    /// Creates a new instance.
    fn new() -> Self;

    /// Advances the game logic by one step.
    fn step(&mut self, core: &mut dyn CoreInterface);
}
