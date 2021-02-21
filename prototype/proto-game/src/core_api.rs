use wasm_bindgen::prelude::*;

pub type RomDataPointer = *const u8;

#[wasm_bindgen]
extern {
    pub type Logger;

    #[wasm_bindgen(method)]
    fn info(this: &Logger, msg: &str);

    #[wasm_bindgen]
    fn another(obj: RomDataPointer);
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

pub fn call_another(endpoint: RomDataRecord) {
    another(endpoint.ptr);
}