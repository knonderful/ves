use crate::mesen::Frame;
use art_extractor_core::geom_art::Size;
use art_extractor_core::movie::{FrameRate, Movie};
use std::path::Path;
use ves_cache::VecCacheMut;

mod mesen;
mod obj;
#[cfg(test)]
pub(crate) mod test_util;

/// Creates a [`Movie`] from the provided Mesen-S export files.
pub fn create_movie(
    files: impl ExactSizeIterator<Item = impl AsRef<Path>>,
) -> anyhow::Result<Movie> {
    let mut palettes = VecCacheMut::new();
    let mut tiles = VecCacheMut::new();

    let mut movie_frames = Vec::with_capacity(files.len());
    for file in files {
        let file_handle = std::fs::File::open(file)?;
        let mesen_frame: Frame = serde_json::from_reader(file_handle)?;
        let movie_frame = obj::create_movie_frame(&mesen_frame, &mut palettes, &mut tiles)?;
        movie_frames.push(movie_frame);
    }

    movie_frames.sort_unstable_by_key(|a| a.frame_number());

    let movie = Movie::new(
        Size::new(512, 256),
        palettes.into_vec(),
        tiles.into_vec(),
        movie_frames,
        FrameRate::Ntsc,
    );
    Ok(movie)
}

#[cfg(test)]
mod test_create_movie {
    use super::create_movie;
    use art_extractor_core::movie::Movie;
    use std::fs::File;
    use ves_cache::SliceCache;

    #[test]
    fn test_full() {
        let mut input_frames_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        input_frames_dir.push("resources/test/mesen-s_frames");

        const NR_OF_FRAMES: usize = 10;

        let mut files = Vec::with_capacity(NR_OF_FRAMES);
        for frame in 0..NR_OF_FRAMES {
            files.push(input_frames_dir.join(format!("frame_{}.json", 199250 + frame)));
        }

        let actual_movie = create_movie(files.iter()).unwrap();
        let palettes = SliceCache::new(actual_movie.palettes());
        let tiles = SliceCache::new(actual_movie.tiles());

        const DEBUG_OUT: bool = false;
        if DEBUG_OUT {
            for frame in actual_movie.frames() {
                let actual = crate::test_util::bmp_from_movie_frame(frame, &palettes, &tiles);
                actual
                    .save(format!(
                        "{}/../../target/movie_frame_{}.bmp",
                        env!("CARGO_MANIFEST_DIR"),
                        frame.frame_number()
                    ))
                    .unwrap();
            }

            let bincode_file = File::create(format!(
                "{}/../../target/movie_{}_frames.bincode",
                env!("CARGO_MANIFEST_DIR"),
                NR_OF_FRAMES
            ))
            .unwrap();
            bincode::serialize_into(bincode_file, &actual_movie).unwrap();

            let json_file = File::create(format!(
                "{}/../../target/movie_{}_frames.json",
                env!("CARGO_MANIFEST_DIR"),
                NR_OF_FRAMES
            ))
            .unwrap();
            serde_json::to_writer(json_file, &actual_movie).unwrap();
            // Alternatively:
            // serde_json::to_writer_pretty(json_file, &movie).unwrap();
        }

        let mut expected_movie_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        expected_movie_path.push(format!(
            "resources/test/expected/movie_{}_frames.bincode",
            NR_OF_FRAMES
        ));
        let expected_movie_file = File::open(expected_movie_path).unwrap();
        let expected_movie: Movie = bincode::deserialize_from(expected_movie_file).unwrap();

        assert_eq!(expected_movie, actual_movie);
    }
}
