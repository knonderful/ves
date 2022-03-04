use ves_art_core::movie::Movie;
use std::io::Write;
use std::path::PathBuf;
use std::fs::File;
use anyhow::Result;

const INPUT_PATH: &'static str = "../../test_movie.bincode";
fn main() -> Result<()> {
    let movie = load_movie_data()?;
    generate_static_code(&movie)?;

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
    const OUTPUT_METHODS_PATH: &'static str = "src/generated/methods.rs";
    let mut generated_methods_file = File::create(OUTPUT_METHODS_PATH)?;
    writeln!(generated_methods_file, "use crate::generated::types::*;")?;
    writeln!(generated_methods_file, "")?;
    writeln!(generated_methods_file, "pub const fn palettes() -> &'static [Palette] {{")?;

    let mut serializer = staticgen::Serializer::new(&mut generated_methods_file);
    use serde::Serialize as _;
    movie.palettes().serialize(&mut serializer)?;

    let structs = std::mem::take(serializer.structs_mut());
    let enums = std::mem::take(serializer.enums_mut());

    writeln!(generated_methods_file, "}}")?;

    const OUTPUT_TYPES_PATH: &'static str = "src/generated/types.rs";
    let mut generated_types_file = File::create(OUTPUT_TYPES_PATH)?;
    structs.write(&mut generated_types_file)?;
    enums.write(&mut generated_types_file)?;

    rust_format::format_file(OUTPUT_TYPES_PATH)?;
    rust_format::format_file(OUTPUT_METHODS_PATH)?;

    Ok(())
}
