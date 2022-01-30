use clap::{Args, Parser, Subcommand};
use std::fs::File;
use std::path::PathBuf;

/// Tool for generating input for Art Extractor from SNES data.
#[derive(Parser, Debug)]
#[clap(version)]
struct SnesCli {
    #[clap(subcommand)]
    command: CliCommand,
}

#[derive(Subcommand, Debug)]
enum CliCommand {
    Movie(MovieArgs),
}

/// Commands related to movies.
#[derive(Args, Debug)]
struct MovieArgs {
    #[clap(subcommand)]
    command: MovieCommand,
}

/// Creates a movie from Mesen-S input files.
#[derive(Subcommand, Debug)]
enum MovieCommand {
    Create(MovieCreateArgs),
}

/// Creates a movie from Mesen-S input files.
#[derive(Args, Debug)]
struct MovieCreateArgs {
    /// The target output file.
    #[clap(name = "out", short = 'o')]
    out_path: String,
    /// The files to use as input (extracted from Mesen-S).
    #[clap(name = "FILES", last = true)]
    in_paths: Vec<String>,
}

fn create_movie(in_paths: &[impl AsRef<str>], out_path: &str) -> anyhow::Result<()> {
    let iter = in_paths
        .iter()
        .map(|in_path| {
            let mut path = PathBuf::new();
            path.push(in_path.as_ref());
            path
        })
        // Below is just a kind of hacky way to show the progress. It presumes that each element in the iterator is consumed and immediately
        // processed (which is not specified by art_extractor_snes::create_movie()... it might collect all paths first and then process them
        // all, in which case this output is more or less bogus.
        .enumerate()
        .map(|(i, path)| {
            println!(
                "Processing file {}/{}: {}",
                i,
                in_paths.len(),
                path.display()
            );
            path
        });

    let movie = art_extractor_snes::create_movie(iter)?;

    println!("Writing output file: {}", out_path);
    let bincode_file = File::create(out_path)?;
    bincode::serialize_into(bincode_file, &movie)?;

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let cli_args: SnesCli = SnesCli::parse();

    match cli_args.command {
        CliCommand::Movie(cmd) => match cmd.command {
            MovieCommand::Create(args) => create_movie(&args.in_paths, &args.out_path)?,
        },
    }

    Ok(())
}
