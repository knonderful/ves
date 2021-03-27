use crate::gpu::{OamTableIndex, OamEntry, OcmTableIndex};
use crate::mem::RomBlock;

/// The interface that must be implemented by a core.
pub trait CoreInterface {

    /// Load graphics data from the game ROM into the object character memory.
    ///
    /// # Parameters
    /// * `index`: The [OcmTableIndex].
    /// * `rom_block`: The [RomBlock].
    fn ocm_load(index: OcmTableIndex, rom_block: RomBlock);

    /// Set an OAM entry.
    ///
    /// # Parameters
    /// * `index`: The [OamTableIndex].
    /// * `oam_entry`: The [OamEntry].
    fn oam_set(index: OamTableIndex, entry: OamEntry);
}

/// The interface that must be implemented by a game.
pub trait GameInterface {

    /// Create a new instance.
    fn new() -> Self;

    /// Advance the game logic by one step.
    fn step(&mut self);
}
