#[repr(C)]
pub struct RomDataEntry<T> {
    data: T,
}

impl<T> RomDataEntry<T> {
    pub const fn new(data: T) -> Self {
        RomDataEntry { data }
    }
}

#[repr(C)]
pub struct RomDataGfx {
    pub mario: RomDataEntry<[u8; 345usize]>,
    pub mario2: RomDataEntry<[u8; 345usize]>,
}

#[repr(C)]
pub struct RomData {
    pub version: (u16, u16, u16),
    pub gfx: RomDataGfx,
}

macro_rules! insert_rom_data {
    () => {
        RomData {
            version: (0, 1, 16),
            gfx: RomDataGfx {
                mario: RomDataEntry::new(*include_bytes!("../assets/gfx/mario.png")),
                mario2: RomDataEntry::new(*include_bytes!("../assets/gfx/mario.png")),
            },
        }
    };
}
