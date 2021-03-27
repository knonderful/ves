use proto_common::gpu::{OamTableEntry, OcmTableIndex, OamTableIndex};
use std::path::Path;
use wasmtime::{Store, Linker, Module, Func, Caller, Extern, Trap, Memory};
use anyhow::Result;
use std::rc::Rc;
use std::cell::{RefCell};
use crate::gfx::{Rgb888, Rectangle2D, Surface, Unit2D, Rgba8888, SliceBackedSurface, RectangleIterator, SliceBackedSurfaceMut, SurfaceValueSet, SurfaceValueGet};
use proto_common::mem::RomBlock;
use proto_common::api::CoreInterface;

/// The width of a character in pixels.
const CHAR_WIDTH: Unit2D = 8;
/// The height of a character in pixels.
const CHAR_HEIGHT: Unit2D = 8;
/// The width of the character table in number of characters.
const OBJ_CHAR_TABLE_WIDTH: Unit2D = 16;
/// The height of the character table in number of characters.
const OBJ_CHAR_TABLE_HEIGHT: Unit2D = 16;
/// The size of the object attribute table in number of entries.
const OBJ_ATTR_MEM_SIZE: usize = 32usize;

// TODO: Replace FrameBufferPixel with another pixel type that only stores the NECESSARY data (basically the indices, not the RGBA)
crate::linear_pixel_buffer!(ObjectCharacterSurfaceBuffer, Rgb888, OBJ_CHAR_TABLE_WIDTH, OBJ_CHAR_TABLE_HEIGHT);

/// A character table.
#[derive(Default)]
struct ObjectCharacterTable {
    surface_buffer: ObjectCharacterSurfaceBuffer,
}

impl ObjectCharacterTable {
    pub fn surface(&self) -> SliceBackedSurface<Rgb888> {
        self.surface_buffer.as_surface()
    }

    pub fn surface_mut(&mut self) -> SliceBackedSurfaceMut<Rgb888> {
        self.surface_buffer.as_surface_mut()
    }

    pub fn obj_rectangle(&self, oam_entry: &OamTableEntry) -> Rectangle2D {
        let char_table_index = oam_entry.char_table_index();
        let origin = (char_table_index.x() as Unit2D * CHAR_WIDTH, char_table_index.y() as Unit2D * CHAR_HEIGHT).into();
        // TODO: Support different sized sprites here
        Rectangle2D::new(origin, (CHAR_WIDTH, CHAR_HEIGHT).into())
    }
}

#[derive(Default)]
struct CoreState {
    obj_char_table: ObjectCharacterTable,
    /// The object attribute table.
    obj_attr_table: [Option<OamTableEntry>; OBJ_ATTR_MEM_SIZE],
}

struct Core {
    rom_data: RomData,
    state: CoreState,
}

impl Core {
    fn new(rom_data: RomData) -> Self {
        Self {
            rom_data,
            state: CoreState::default(),
        }
    }
}

impl CoreInterface for Core {
    fn log_info(&self, message: &str) {
        println!("[GAME:INFO] {}", message);
    }

    fn ocm_load(&mut self, index: OcmTableIndex, rom_block: RomBlock) {
        let x = index.x() as Unit2D * 8;
        let y = index.y() as Unit2D * 8;

        let len = rom_block.len();
        assert_eq!(len, 8 * 8 * 3); // 3 bytes per pixel

        let record_slice = self.rom_data.slice(rom_block);
        let src_surf = SliceBackedSurface::<Rgb888>::new(record_slice, (8, 8).into());

        let mut dest_surf = self.state.obj_char_table.surface_mut();

        let src_iter = RectangleIterator::new(src_surf.dimensions());
        let dest_rect = Rectangle2D::new((x, y).into(), src_surf.dimensions());
        let dest_iter = RectangleIterator::new_with_rectangle(dest_surf.dimensions(), dest_rect);

        src_iter.zip(dest_iter).for_each(|(src_pos, dest_pos)| {
            dest_surf.set_value(dest_pos, &src_surf.get_value(src_pos));
        });
    }

    fn oam_set(&mut self, index: OamTableIndex, entry: OamTableEntry) {
        self.state.obj_attr_table[u8::from(index) as usize] = Some(entry);
    }
}

pub struct CoreAndGame {
    instance_ptr: u32,
    step: Func,
    core: Rc<RefCell<Core>>,
}

fn get_memory(caller: &Caller<'_>) -> std::result::Result<Memory, Trap> {
    match caller.get_export("memory") {
        Some(Extern::Memory(mem)) => Ok(mem),
        _ => Err(Trap::new("Failed to find memory.")),
    }
}

fn get_slice(mem: &Memory, ptr: u32, len: u32) -> std::result::Result<&[u8], Trap> {
    unsafe {
        mem.data_unchecked()
            .get(ptr as u32 as usize..)
            .and_then(|arr| arr.get(..len as usize))
            .ok_or_else(|| Trap::new(format!("Could not get slice with pointer {} and length {}.", ptr, len)))
    }
}

fn get_str(data: &[u8]) -> std::result::Result<&str, Trap> {
    match std::str::from_utf8(data) {
        Ok(str) => Ok(str),
        Err(_) => Err(Trap::new("Invalid UTF-8")),
    }
}

impl CoreAndGame {
    pub fn from_path(path: &Path) -> Result<CoreAndGame> {
        let wasm_file = std::fs::canonicalize(path)?;
        let rom_data = RomData::from_path(&wasm_file)?;

        let store = Store::default();
        let module = Module::from_file(store.engine(), &wasm_file)?;

        let mut linker = Linker::new(&store);

        // Use RefCell for internal mutability, since we need to create the import functions first
        // and only then we can create the CoreAndGame, since we need the exported functions for that.
        // Since the import functions only support Fn(...), we can not use a mutable reference in
        // the implementations and thus we need to use RefCell's internal mutability.
        // We could use a Mutex, but that would just introduce synchronization overhead that we
        // can avoid, since everything is running in one thread.
        let core = Rc::new(RefCell::new(Core::new(rom_data)));

        {
            let core_clone = core.clone();
            linker.func("oam", "set", move |index: u32, oam_entry: u32| {
                core_clone.borrow_mut()
                    .oam_set((index as u8).into(), oam_entry.into());
            })?;
        }

        {
            let core_clone = core.clone();
            linker.func("ocm", "load", move |index: u32, rom_block: u64| {
                core_clone.borrow_mut()
                    .ocm_load((index as u8).into(), rom_block.into());
            })?;
        }

        {
            let core_clone = core.clone();
            linker.func("logger", "info", move |caller: Caller<'_>, ptr: u32, len: u32| {
                let mem = get_memory(&caller)?;
                let message = get_str(get_slice(&mem, ptr, len)?)?;

                core_clone.borrow_mut()
                    .log_info(message);

                Ok(())
            })?;
        }

        let instance = linker.instantiate(&module)?;

        let create_instance = instance
            .get_func("create_instance")
            .ok_or(anyhow::format_err!("failed to find `create_instance` function export"))?
            .get0::<u32>()?;

        let instance_ptr = create_instance()?;

        let step = instance
            .get_func("step")
            .ok_or(anyhow::format_err!("failed to find `step` function export"))?;

        let core_and_game = CoreAndGame {
            instance_ptr,
            step,
            core,
        };

        Ok(core_and_game)
    }

    pub(crate) fn step(&self) {
        let function = self.step.get1::<u32, ()>().unwrap();
        function(self.instance_ptr).unwrap();
    }

    pub(crate) fn render(&self, framebuffer: &mut SliceBackedSurfaceMut<Rgba8888>) {
        // Fill background with one color
        let bg_color = (0, 64, 0, 255).into();

        RectangleIterator::new(framebuffer.dimensions()).for_each(|pos| {
            framebuffer.set_value(pos, &bg_color);
        });

        let internal = self.core.borrow();
        let state = &internal.state;
        let obj_char_table = &state.obj_char_table;
        let obj_char_surface = &obj_char_table.surface();

        // Use this color for transparency
        let transparent = (255, 0, 255).into();

        for sprite_opt in state.obj_attr_table.iter() {
            if let Some(sprite) = sprite_opt {
                let sprite_rect = obj_char_table.obj_rectangle(&sprite);
                let src_iter = RectangleIterator::new_with_rectangle(obj_char_surface.dimensions(), sprite_rect);

                let (x, y) = sprite.position();
                let dest_rect = Rectangle2D::new((x as _, y as _).into(), sprite_rect.dimensions);
                let dest_iter = RectangleIterator::new_with_rectangle(framebuffer.dimensions(), dest_rect);

                src_iter.zip(dest_iter).for_each(|(src_pos, dest_pos)| {
                    let src_value = obj_char_surface.get_value(src_pos);
                    if src_value == transparent {
                        return;
                    }

                    // TODO: Perform conversion with dedicated structs or something.
                    //       Think about lossy vs non-lossy and how that should be reflected in the code.
                    let dest_value = Rgba8888 {
                        r: src_value.r,
                        g: src_value.g,
                        b: src_value.b,
                        a: 255,
                    };

                    framebuffer.set_value(dest_pos, &dest_value);
                });
            }
        }
    }
}

/// ROM data.
///
/// This is usually a custom section in the WASM binary and contains assets for the game that are to
/// be used by the core, such as graphics and sound data. Such assets are normally not mutable or
/// generated at run-time and as such do not need to cross the WASM ABI. A game implementation can
/// pass references to parts of the ROM data to the core (essentially an offset and a size). Such a
/// reference is called a [RomDataRecord].
pub struct RomData {
    data: Vec<u8>,
}

impl RomData {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    pub fn from_path(path: impl AsRef<Path>) -> Result<RomData> {
        const ROM_DATA: &str = "rom_data";

        let module = parity_wasm::deserialize_file(&path)?;
        let payload = module
            .custom_sections()
            .find(|sect| sect.name() == ROM_DATA)
            .ok_or(anyhow::Error::msg(format!("Could not find rom data (custom section '{}') in {}.", ROM_DATA, path.as_ref().display())))?
            .payload();
        Ok(Self::new(Vec::from(payload)))
    }

    pub fn slice(&self, rom_block: RomBlock) -> &[u8] {
        let start = rom_block.offset() as usize;
        let end = start + rom_block.len() as usize;
        &self.data[start..end]
    }
}
