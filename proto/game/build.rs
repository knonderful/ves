use art_extractor_core::movie::Movie;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    const INPUT_PATH: &'static str = "../../test_movie.bincode";

    use std::fs::File;
    let movie_file_path = PathBuf::from(INPUT_PATH);
    let movie_file = File::open(&movie_file_path).unwrap();
    let movie: Movie = bincode::deserialize_from(movie_file).unwrap();

    const OUTPUT_METHODS_PATH: &'static str = "src/generated/methods.rs";
    let mut generated_methods_file = File::create(OUTPUT_METHODS_PATH).unwrap();
    writeln!(generated_methods_file, "use crate::generated::types::*;").unwrap();
    writeln!(generated_methods_file, "").unwrap();
    writeln!(generated_methods_file, "pub const fn palettes() -> &'static [Palette] {{").unwrap();

    let mut serializer = staticgen::Serializer::new(&mut generated_methods_file);
    use serde::Serialize as _;
    movie.palettes().serialize(&mut serializer).unwrap();

    let structs = std::mem::take(serializer.structs_mut());
    let enums = std::mem::take(serializer.enums_mut());

    writeln!(generated_methods_file, "}}").unwrap();

    const OUTPUT_TYPES_PATH: &'static str = "src/generated/types.rs";
    let mut generated_types_file = File::create(OUTPUT_TYPES_PATH).unwrap();
    structs.write(&mut generated_types_file).unwrap();
    enums.write(&mut generated_types_file).unwrap();

    rust_format::format_file(OUTPUT_TYPES_PATH).unwrap();
    rust_format::format_file(OUTPUT_METHODS_PATH).unwrap();

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={INPUT_PATH}");
}
