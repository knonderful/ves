pub type RomDataPointer = *const u8;

#[link(wasm_import_module = "gpu")]
extern {
    #[link_name = "set_object"]
    fn gpu_set_object(index: u16, ptr: RomDataPointer, size: usize);
}

#[link(wasm_import_module = "logger")]
extern {
    #[link_name = "info"]
    fn logger_info(ptr: *const u8, len: usize);
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

    pub fn info(&mut self, message: &String) {
        unsafe {
            logger_info(message.as_ptr(), message.len());
        }
    }
}

pub struct Gpu {
    pub objects: Objects,
}

impl Gpu {
    fn new() -> Self {
        Self {
            objects: Objects::new(),
        }
    }
}

pub struct Objects {}

impl Objects {
    fn new() -> Self {
        Self {}
    }

    pub fn set(&mut self, index: u16, record: RomDataRecord) {
        unsafe { gpu_set_object(index, record.ptr, record.size) };
    }
}