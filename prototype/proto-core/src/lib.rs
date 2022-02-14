#![allow(clippy::zero_ptr)] // Need this for the libretro_core!() macro
mod core;
mod libretro_core;
#[macro_use]
mod gfx;

#[macro_use]
extern crate libretro_backend;

libretro_core!(libretro_core::LibretroProtoCore);
