use proto_common::gpu::{OamEntry, OamTableIndex};

pub type RomDataPointer = *const u8;

#[link(wasm_import_module = "logger")]
extern {
    #[link_name = "info"]
    fn logger_info(ptr: *const u8, len: usize);
}

#[repr(u8)]
pub enum ObjectSize {
    Size8x8 = 0,
    Size16x16 = 1,
}

#[link(wasm_import_module = "obj_char_mem")]
extern {
    #[link_name = "load"]
    fn obj_char_mem_load(x: u8, y: u8, ptr: RomDataPointer, size: ObjectSize);
}

#[link(wasm_import_module = "obj_attr_mem")]
extern {
    /// Set an OAM entry.
    ///
    /// # Parameters:
    /// * `index`: An [OamTableIndex] as an `u8`.
    /// * `oam_entry`: An [OamEntry] as an `u32`.
    #[link_name = "set"]
    fn obj_attr_mem_set(index: u8, oam_entry: u32);
}

#[allow(dead_code)]
pub struct RomDataRecord {
    ptr: RomDataPointer,
    size: usize,
}

impl RomDataRecord {
    pub fn new(ptr: RomDataPointer, size: usize) -> Self {
        Self { ptr, size }
    }

    pub fn ptr(&self) -> RomDataPointer {
        self.ptr
    }

    #[allow(dead_code)]
    pub fn size(&self) -> usize {
        self.size
    }
}

pub struct Core {
    pub logger: Logger,
    pub gpu: Gpu,
}

impl Core {
    pub fn new() -> Self {
        Self {
            logger: Logger::new(),
            gpu: Gpu::new(),
        }
    }
}

pub struct Logger {}

impl Logger {
    fn new() -> Self {
        Self {}
    }

    pub fn info(&mut self, message: &str) {
        unsafe {
            logger_info(message.as_ptr(), message.len());
        }
    }
}

pub struct Gpu {
    pub objects: Objects,
    pub char_table: ObjectCharacterTable,
}

impl Gpu {
    fn new() -> Self {
        Self {
            objects: Objects::new(),
            char_table: ObjectCharacterTable::new(),
        }
    }
}

pub struct ObjectCharacterTable {}

impl ObjectCharacterTable {
    fn new() -> Self {
        Self {}
    }

    pub fn load(&mut self, index: ObjectCharacterTableIndex, ptr: RomDataPointer, size: ObjectSize) {
        unsafe { obj_char_mem_load(index.x, index.y, ptr, size) }
    }
}

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

impl From<(u8, u8)> for ObjectCharacterTableIndex {
    fn from(val: (u8, u8)) -> Self {
        Self { x: val.0, y: val.1 }
    }
}

pub struct Objects {}

impl Objects {
    fn new() -> Self {
        Self {}
    }

    pub fn set(&mut self, index: &OamTableIndex, entry: &OamEntry) {
        unsafe { obj_attr_mem_set(index.into(), entry.into()) };
    }
}