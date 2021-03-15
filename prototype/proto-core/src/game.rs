use crate::game_api::{RomData, RomDataRecord};

use std::path::Path;
use wasmtime::{Store, Linker, Module, Func, Caller, Extern, Trap, Memory};
use anyhow::Result;
use std::rc::Rc;
use std::cell::{RefCell};
use crate::gfx::{Position2D, Rgb888, Rectangle2D, Surface, Unit2D, Rgba8888, SliceBackedSurface, RectangleIterator, SliceBackedSurfaceMut, SurfaceValueSet, SurfaceValueGet};
use std::ops::DerefMut;

// TODO: Copied from proto-game. Needs unifying.
#[derive(Copy, Clone)]
pub struct ObjectCharacterTableIndex {
    x: u8,
    y: u8,
}

impl ObjectCharacterTableIndex {
    pub fn new(x: u8, y: u8) -> Self {
        Self { x, y }
    }
}

#[derive(Copy, Clone)]
pub struct SpriteObject {
    char_table_index: ObjectCharacterTableIndex,
    position: Position2D,
}

impl SpriteObject {
    fn new(char_table_index: ObjectCharacterTableIndex, position: Position2D) -> Self {
        Self {
            char_table_index,
            position,
        }
    }
}

/// The width of a character in pixels.
const CHAR_WIDTH: usize = 8;
/// The height of a character in pixels.
const CHAR_HEIGHT: usize = 8;
/// The size of a character in pixels.
const CHAR_SIZE: usize = CHAR_WIDTH * CHAR_HEIGHT;
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
}

#[derive(Default)]
struct GameState {
    obj_char_table: ObjectCharacterTable,
    /// The object attribute table.
    obj_attr_table: [Option<SpriteObject>; OBJ_ATTR_MEM_SIZE],
}

struct GameInternal {
    rom_data: RomData,
    state: GameState,
}

impl GameInternal {
    fn new(rom_data: RomData) -> Self {
        Self {
            rom_data,
            state: GameState::default(),
        }
    }
}

pub struct Game {
    instance_ptr: u32,
    step: Func,
    internal: Rc<RefCell<GameInternal>>,
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

impl Game {
    pub fn from_path(path: &Path) -> Result<Game> {
        let wasm_file = std::fs::canonicalize(path)?;
        let rom_data = get_rom_data(&wasm_file)?;

        // Use RefCell for internal mutability, since we need to create the import functions first
        // and only then we can create the Game, since we need the exported functions for that.
        // Since the import functions only support Fn(...), we can not use a mutable reference in
        // the implementations and thus we need to use RefCell's internal mutability.
        // We could use a Mutex, but that would just introduce synchronization overhead that we
        // can avoid, since everything is running in one thread.
        let game_internal = Rc::new(RefCell::new(GameInternal::new(rom_data)));

        let store = Store::default();
        let module = Module::from_file(store.engine(), &wasm_file)?;

        let mut linker = Linker::new(&store);

        let game_int = game_internal.clone();

        linker.func("obj_attr_mem", "set", move |index: u32, ocm_x: u32, ocm_y: u32| {
            let char_mem_index = ObjectCharacterTableIndex::new(ocm_x as u8, ocm_y as u8);
            let mut game_int = (*game_int).borrow_mut();
            game_int.state.obj_attr_table[index as usize] = Some(SpriteObject::new(char_mem_index, Position2D::new(0, 0)));
        })?;

        let game_int = game_internal.clone();
        linker.func("obj_char_mem", "load", move |x: u32, y: u32, ptr: u32, size: u32| {
            // We don't support other sizes yet.
            assert_eq!(0, size);

            let x = x as Unit2D * 8;
            let y = y as Unit2D * 8;

            let mut game_int = (*game_int).borrow_mut();
            let game_int = game_int.deref_mut();

            let len = 8 * 8 * 3; // 3 bytes per pixel
            let record = game_int.rom_data.record(ptr, len);
            let record_slice = record.slice(&game_int.rom_data);
            let src_surf = SliceBackedSurface::<Rgb888>::new(record_slice, (8, 8).into());

            let mut dest_surf = game_int.state.obj_char_table.surface_mut();

            let src_iter = RectangleIterator::new(src_surf.dimensions());
            let dest_rect = Rectangle2D::new((x, y).into(), src_surf.dimensions());
            let dest_iter = RectangleIterator::new_with_rectangle(dest_surf.dimensions(), dest_rect);

            src_iter.zip(dest_iter).for_each(|(src_pos, dest_pos)| {
                dest_surf.set_value(dest_pos, &src_surf.get_value(src_pos));
            });
        })?;

        linker.func("logger", "info", |caller: Caller<'_>, ptr: u32, len: u32| {
            let mem = get_memory(&caller)?;
            let message = get_str(get_slice(&mem, ptr, len)?)?;
            println!("[GAME:INFO] {}", message);
            Ok(())
        })?;

        let instance = linker.instantiate(&module)?;

        let create_instance = instance
            .get_func("create_instance")
            .ok_or(anyhow::format_err!("failed to find `create_instance` function export"))?
            .get0::<u32>()?;

        let instance_ptr = create_instance()?;

        let step = instance
            .get_func("step")
            .ok_or(anyhow::format_err!("failed to find `step` function export"))?;

        let game = Game {
            instance_ptr,
            step,
            internal: game_internal,
        };

        Ok(game)
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

        let internal = self.internal.borrow();
        let state = &internal.state;
        let obj_chars = &state.obj_char_table;

        for sprite_opt in state.obj_attr_table.iter() {
            if let Some(sprite) = sprite_opt {

                // let it_src = sprite.record.slice(rom_data).chunks_exact(3).map(|chunk| (chunk[0], chunk[1], chunk[2]));
                // let it_dest = framebuffer.window(Rectangle::new(Position::new(32, 64), Dimensions::new(8, 8)));
                //
                // it_dest.zip(it_src).for_each(|(dst, src)| {
                //     if src != TRANSPARENT {
                //         dst.set_rgb(src.0, src.1, src.2);
                //     }
                // });
            }
        }
    }
}

fn get_rom_data(path: impl AsRef<Path>) -> Result<RomData> {
    const ROM_DATA: &str = "rom_data";

    let module = parity_wasm::deserialize_file(&path)?;
    let payload = module
        .custom_sections()
        .find(|sect| sect.name() == ROM_DATA)
        .ok_or(anyhow::Error::msg(format!("Could not find rom data (custom section '{}') in {}.", ROM_DATA, path.as_ref().display())))?
        .payload();
    Ok(RomData::new(Vec::from(payload)))
}