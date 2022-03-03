use art_extractor_core::movie::Movie;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    const INPUT_PATH: &'static str = "../../test_movie.bincode";

    use std::fs::File;
    let movie_file_path = PathBuf::from(INPUT_PATH);
    let movie_file = File::open(&movie_file_path).unwrap();
    let movie: Movie = bincode::deserialize_from(movie_file).unwrap();

    let mut generated_methods_file = File::create("src/generated/methods.rs").unwrap();
    writeln!(generated_methods_file, "use crate::generated::types::*;").unwrap();
    writeln!(generated_methods_file, "").unwrap();
    writeln!(generated_methods_file, "pub const fn palettes() -> &'static [Palette] {{").unwrap();

    let mut serializer = staticgen::Serializer::new(&mut generated_methods_file);
    use serde::Serialize as _;
    movie.palettes().serialize(&mut serializer).unwrap();

    let structs = serializer.take_structs();
    let enums = serializer.take_enums();

    writeln!(generated_methods_file, "}}").unwrap();

    let mut generated_types_file = File::create("src/generated/types.rs").unwrap();
    structs.write(&mut generated_types_file).unwrap();
    enums.write(&mut generated_types_file).unwrap();

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={INPUT_PATH}");
}
