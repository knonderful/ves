use std::path::{Path, PathBuf};
use art_extractor_core::geom_art::Size;
use art_extractor_core::movie::{FrameRate, Movie};
use ves_cache::IndexedCache;
use crate::mesen::Frame;

#[cfg(test)]
pub(crate) mod test_util;
mod mesen;
mod obj;

fn json_files(directory: &impl AsRef<Path>) -> std::io::Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    for dir_entry_result in std::fs::read_dir(directory)? {
        let path = dir_entry_result?.path();
        if path.is_file() && path.ends_with(".json") {
            paths.push(path);
        }
    }
    Ok(paths)
}

/// Creates a [`Movie`] from the Mesen-S export files in the provided directory.
pub fn create_movie(directory: impl AsRef<Path>) -> anyhow::Result<Movie> {
    // Get all files, filter on JSON
    let mut files = json_files(&directory)?;
    // Sort by file name
    files.sort_unstable();

    let mut palettes = IndexedCache::new();
    let mut tiles = IndexedCache::new();

    let mut movie_frames = Vec::with_capacity(files.len());
    for file in files {
        let file_handle = std::fs::File::open(file)?;
        let mesen_frame: Frame = serde_json::from_reader(file_handle)?;
        let movie_frame = obj::create_movie_frame(&mesen_frame, &mut palettes, &mut tiles)?;
        movie_frames.push(movie_frame);
    }

    let movie = Movie::new(Size::new(512.into(), 256.into()), palettes.consume(), tiles.consume(), movie_frames, FrameRate::Ntsc);
    Ok(movie)
}