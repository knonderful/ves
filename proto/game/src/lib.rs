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
pub static ROM_DATA: [u8; 983752] = *include_bytes!(concat!(env!("OUT_DIR"), "/vrom.bincode"));

static PALETTES: &'static [crate::generated::types::Palette] =
    crate::generated::methods::palettes();

static FRAMES: &'static [crate::generated::types::MovieFrame] = crate::generated::methods::frames();

pub struct ProtoGame {
    core: CoreBootstrap,
    frame_nr: usize,
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
        Self { core, frame_nr: 0 }
    }

    fn step(&mut self) {
        info!("Game frame number: {}.", self.frame_nr);

        // Upload all palettes on the first frame
        if self.frame_nr == 0 {
            info!("Uploading all palettes.");
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

        let movie_frame = &FRAMES[self.frame_nr % FRAMES.len()];
        info!("Uploading movie frame #{}.", movie_frame.frame_number);

        for (i, sprite) in movie_frame.sprites.iter().enumerate() {
            let entry = OamTableEntry::new(
                from_unchecked(sprite.position.x.0),
                from_unchecked(sprite.position.y.0),
                from_unchecked(sprite.palette.0),
                u8::from(sprite.h_flip),
                u8::from(sprite.v_flip),
                from_unchecked(sprite.tile.0),
            );
            self.core.oam_set(&OamTableIndex::new(from_unchecked(i)), &entry);
        }

        self.frame_nr += 1;
    }
}

ves_proto_common::create_game!(ProtoGame);
