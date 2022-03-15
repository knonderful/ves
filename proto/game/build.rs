use anyhow::{anyhow, Result};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use ves_art_core::movie::Movie;

const INPUT_PATH: &str = "../../test_movie.bincode";
fn main() -> Result<()> {
    let movie = load_movie_data()?;
    generate_static_code(&movie)?;
    generate_vrom_data(&movie)?;

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={INPUT_PATH}");

    Ok(())
}

fn load_movie_data() -> Result<Movie> {
    let movie_file_path = PathBuf::from(INPUT_PATH);
    let movie_file = File::open(&movie_file_path)?;
    Ok(bincode::deserialize_from(movie_file)?)
}

fn generate_static_code(movie: &Movie) -> Result<()> {
    const OUTPUT_METHODS_PATH: &str = "src/generated/methods.rs";
    let generated_methods_file = File::create(OUTPUT_METHODS_PATH)?;
    let mut serializer = staticgen::Serializer::new(generated_methods_file);
    writeln!(serializer.out_mut(), "use crate::generated::types::*;")?;
    writeln!(serializer.out_mut())?;
    writeln!(
        serializer.out_mut(),
        "pub const fn palettes() -> &'static [Palette] {{"
    )?;

    use serde::Serialize as _;
    movie.palettes().serialize(&mut serializer)?;

    writeln!(serializer.out_mut(), "}}")?;
    writeln!(serializer.out_mut())?;
    writeln!(
        serializer.out_mut(),
        "pub const fn frames() -> &'static [MovieFrame] {{"
    )?;

    let frames = movie
        .frames()
        .chunks(10)
        .next()
        .ok_or_else(|| anyhow!("Got no frames."))?;
    frames.serialize(&mut serializer)?;

    writeln!(serializer.out_mut(), "}}")?;

    let structs = std::mem::take(serializer.structs_mut());
    let enums = std::mem::take(serializer.enums_mut());

    const OUTPUT_TYPES_PATH: &str = "src/generated/types.rs";
    let mut generated_types_file = File::create(OUTPUT_TYPES_PATH)?;
    writeln!(&mut generated_types_file, "#![allow(clippy::all)]")?;
    structs.write(&mut generated_types_file)?;
    enums.write(&mut generated_types_file)?;

    rust_format::format_file(OUTPUT_TYPES_PATH)?;
    rust_format::format_file(OUTPUT_METHODS_PATH)?;

    Ok(())
}

fn generate_vrom_data(movie: &Movie) -> Result<()> {
    let mut path = PathBuf::new();
    path.push(std::env::var("OUT_DIR")?);
    path.push("vrom.bincode");

    let mut vrom_file = File::create(&path)?;
    bincode::serialize_into(&mut vrom_file, movie.tiles())?;
    Ok(())
}
