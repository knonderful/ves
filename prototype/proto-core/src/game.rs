use crate::game_api::{RomData, RomDataRecord};
use crate::{FrameBuffer, FrameBufferPixel};

use std::path::Path;
use wasmtime::{Store, Linker, Module, Func};
use anyhow::Result;
use std::rc::Rc;
use std::cell::{RefCell};

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
    greet: Func,
    internal: Rc<RefCell<GameInternal>>,
}

impl Game {
    pub fn from_path(path: &Path) -> Result<Game>  {
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

        let instance = linker.instantiate(&module)?;

        let hello = instance
            .get_func("greet")
            .ok_or(anyhow::format_err!("failed to find `greet` function export"))?;

        let game = Game {
            greet: hello,
            internal: game_internal,
        };

        Ok(game)
    }

    pub(crate) fn step(&self) {
        let function = self.greet.get0::<()>().unwrap();
        function().unwrap();
    }

    pub(crate) fn render(&self, framebuffer: &mut FrameBuffer) {
        let internal = self.internal.borrow();
        let rom_data = &internal.rom_data;
        let state = &internal.state;

        let transparent: FrameBufferPixel = (255, 0, 255).into();

        for sprite_opt in state.obj_table.iter() {
            if let Some(sprite) = sprite_opt {
                let data = sprite.record.slice(rom_data);
                let mut cursor = framebuffer.cursor();
                let mut x = 0;
                let mut y = 0;
                cursor.move_to(x, y);
                for chunk in data.chunks_exact(3) {
                    if x % 8 == 0 {
                        x = 0;
                        y = y + 1;
                        cursor.move_to(0, y);
                    }

                    x = x + 1;

                    let color = (chunk[0], chunk[1], chunk[2]).into();
                    if color != transparent {
                        cursor.set_pixel(color);
                    }

                    cursor.advance();
                }
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