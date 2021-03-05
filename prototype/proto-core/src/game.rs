use crate::game_api::{RomData, RomDataRecord};
use crate::core::{FrameBuffer, Pixel};

use std::path::Path;
use wasmtime::{Store, Linker, Module, Func, Caller, Extern, Trap, Memory};
use anyhow::Result;
use std::rc::Rc;
use std::cell::{RefCell};
use crate::core::geometry::{Rectangle, Position, Dimensions};

pub struct SpriteObject {
    record: RomDataRecord,
}

impl SpriteObject {
    fn new(record: RomDataRecord) -> Self {
        Self {
            record
        }
    }
}

#[derive(Default)]
struct GameState {
    obj_table: [Option<SpriteObject>; 32usize], // 32-sprite limit
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

        linker.func("gpu", "set_object", move |index: u32, ptr: u32, size: u32| {
            let mut game_int = (*game_int).borrow_mut();
            let record = game_int.rom_data.record(ptr, size);
            game_int.state.obj_table[index as usize] = Some(SpriteObject::new(record));
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

    pub(crate) fn render(&self, framebuffer: &mut FrameBuffer) {
        let internal = self.internal.borrow();
        let rom_data = &internal.rom_data;
        let state = &internal.state;

        // Fill background with one color
        framebuffer.window(Rectangle::new(Position::origin(), Dimensions::new(framebuffer.width(), framebuffer.height())))
            .for_each(|pixel| {
                pixel.set_rgb(0, 64, 0);
            });

        const TRANSPARENT: (u8, u8, u8) = (255, 0, 255);

        for sprite_opt in state.obj_table.iter() {
            if let Some(sprite) = sprite_opt {
                let it_src = sprite.record.slice(rom_data).chunks_exact(3).map(|chunk| (chunk[0], chunk[1], chunk[2]));
                let it_dest = framebuffer.window(Rectangle::new(Position::new(32, 64), Dimensions::new(8, 8)));

                it_dest.zip(it_src).for_each(|(dst, src)| {
                    if src != TRANSPARENT {
                        dst.set_rgb(src.0, src.1, src.2);
                    }
                });
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