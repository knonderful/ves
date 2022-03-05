mod generated;

use log::info;
use ves_proto_common::api::{Core, CoreBootstrap, Game};
use ves_proto_common::gpu::{
    OamTableEntry, OamTableIndex, PaletteColor, PaletteIndex, PaletteTableIndex,
};

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// This will be used by the Core to grab graphics data like tiles.
#[allow(dead_code)]
#[link_section = "vrom"]
static ROM_DATA: [u8; 983752] = *include_bytes!(concat!(env!("OUT_DIR"), "/vrom.bincode"));

static PALETTES: &'static [crate::generated::types::Palette] =
    crate::generated::methods::palettes();

pub struct ProtoGame {
    core: CoreBootstrap,
    frame_nr: u32,
}

fn from_unchecked<A, B>(a: A) -> B
where
    B: TryFrom<A>,
    <B as TryFrom<A>>::Error: std::fmt::Debug,
{
    TryFrom::try_from(a).unwrap()
}

impl Game for ProtoGame {
    fn new(core: CoreBootstrap) -> Self {
        Self {
            core,
            frame_nr: 0,
        }
    }

    fn step(&mut self) {
        self.frame_nr += 1;
        info!("At frame {}", self.frame_nr);

        if self.frame_nr == 1 {
            for (pal_idx, palette) in PALETTES.iter().enumerate() {
                for (col_idx, color) in palette.colors.iter().enumerate() {
                    use crate::generated::types::Color;
                    let color = match color {
                        Color::Opaque(rgb) => PaletteColor::new(rgb.r, rgb.g, rgb.b),
                        Color::Transparent => PaletteColor::new(0, 0, 0),
                    };

                    let palette = PaletteTableIndex::new(from_unchecked(pal_idx));
                    let index = PaletteIndex::new(from_unchecked(col_idx));
                    self.core.palette_set(&palette, &index, &color);
                }
            }
        }

        let index = OamTableIndex::new(0);
        let entry = OamTableEntry::new(10, 20, 3, 1, 0, 123);
        self.core.oam_set(&index, &entry);

        let palette = PaletteTableIndex::new(2);
        let index = PaletteIndex::new(14);
        let color = PaletteColor::new(3, 2, 1);
        self.core.palette_set(&palette, &index, &color);
    }
}

ves_proto_common::create_game!(ProtoGame);