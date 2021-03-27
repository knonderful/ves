mod game_api;
mod core;
mod libretro_core;
#[macro_use]
mod gfx;

#[macro_use]
extern crate libretro_backend;

libretro_core!( libretro_core::LibretroProtoCore );