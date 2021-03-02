mod game_api;
mod game;
mod core;

#[macro_use]
extern crate libretro_backend;

libretro_core!( core::ProtoCore );