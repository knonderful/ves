pub type RomDataPointer = *const u8;

#[link(wasm_import_module = "gpu")]
extern {
    #[link_name = "set_object"]
    fn gpu_set_object(index: usize, ptr: RomDataPointer);
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

pub struct Gpu {}

impl Gpu {
    pub fn set_object(&self, index: usize, record: RomDataRecord) {
        unsafe { gpu_set_object(index, record.ptr) };
    }
}

static GPU: Gpu = Gpu {};

pub fn gfx() -> &'static Gpu {
    &GPU
}