use crate::gpu::{OamTableEntry, OamTableIndex, PaletteColor, PaletteIndex, PaletteTableIndex};

/// The prototype core API.
pub trait Core {
    /// Sets an OAM entry.
    ///
    /// # Arguments
    ///
    /// * `index`: The index into the OAM table.
    /// * `entry`: The entry.
    fn oam_set(&self, index: &OamTableIndex, entry: &OamTableEntry);

    /// Sets a palette entry.
    ///
    /// # Arguments
    ///
    /// * `palette`: The index of the palette in the palette table.
    /// * `index`: The index inside the palette.
    /// * `color`: The color to set.
    fn palette_set(&self, palette: &PaletteTableIndex, index: &PaletteIndex, color: &PaletteColor);
}

/// The prototype game API.
pub trait Game {
    /// Create a game instance.
    ///
    /// # Arguments
    ///
    /// * `core`: The bootstrap to the core API. This instance should be used by the game
    ///           implementation to interact with the core.
    fn new(core: CoreBootstrap) -> Self;

    /// Advance the game by one step.
    fn step(&mut self);
}

pub struct CoreBootstrap {
    core_gpu_oam_set: unsafe extern "C" fn(index: u8, entry: u64),
    core_gpu_palette_set: unsafe extern "C" fn(palette: u8, index: u8, color: u16),
}

/// A helper for bootstrapping the core to the game code.
///
/// See [`Game`] for the core-side API.
impl CoreBootstrap {
    /// Creates a new [`CoreBootstrap`].
    ///
    /// This function is normally not called directly, but instead used by the `create_game!()`
    /// macro.
    ///
    /// # Arguments
    ///
    /// * `core_log_log`: The pointer to the `log::log()` function.
    /// * `core_gpu_oam_set`: The pointer to the `gpu::oam_set()` function.
    /// * `core_gpu_palette_set`: The pointer to the `gpu::palette_set()` function.
    /// * `log_init`: A callback for initializing the logger.
    pub fn new(
        core_log_log: unsafe extern "C" fn(level: u32, ptr: *const u8, len: usize),
        core_gpu_oam_set: unsafe extern "C" fn(index: u8, entry: u64),
        core_gpu_palette_set: unsafe extern "C" fn(palette: u8, index: u8, color: u16),
        log_init: impl FnOnce(
            unsafe extern "C" fn(level: u32, ptr: *const u8, len: usize),
        ) -> Result<(), String>,
    ) -> Self {
        log_init(core_log_log).unwrap();

        Self {
            core_gpu_oam_set,
            core_gpu_palette_set,
        }
    }
}

impl Core for CoreBootstrap {
    fn oam_set(&self, index: &OamTableIndex, entry: &OamTableEntry) {
        unsafe {
            (self.core_gpu_oam_set)(index.into(), entry.into());
        }
    }

    fn palette_set(&self, palette: &PaletteTableIndex, index: &PaletteIndex, color: &PaletteColor) {
        unsafe {
            (self.core_gpu_palette_set)(palette.into(), index.into(), color.into());
        }
    }
}

/// A macro for bootstrapping a game implementation.
///
/// # Arguments
///
/// * `$game`: The name of the type that implements the [`Game`] trait.
///
/// # Examples
///
/// ```ignore
/// use ves_proto_common::api::{CoreBootstrap, Game};
///
/// struct GameImpl {
///     core: CoreBootstrap,
/// }
///
/// impl Game for GameImpl {
///     fn new(core: CoreBootstrap) -> Self {
///         Self { core }
///     }
///
///     fn step(&mut self) {
///         ves_proto_common::api::Core as _;
///         // Call core API, e.g.:
///         // self.core.oam_set(.....);
///     }
/// }
///
/// // Make sure that the game crate has `ves-proto-logger` as a dependency.
/// ves_proto_common::create_game!(GameImpl);
/// ```
#[macro_export]
macro_rules! create_game {
    ($game:ty) => {
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

        #[no_mangle]
        pub fn create_instance() -> Box<$game> {
            let core = CoreBootstrap::new(
                core_log_log,
                core_gpu_oam_set,
                core_gpu_palette_set,
                |cll| {
                    ves_proto_logger::Logger::new(core_log_log)
                        .init(Some(ves_proto_common::log::LogLevel::Trace))
                        .map_err(|err| String::from("Could not set logger."))
                },
            );
            let game = <$game>::new(core);
            Box::new(game)
        }

        #[no_mangle]
        pub fn step(game: &mut $game) {
            game.step();
        }
    };
}
