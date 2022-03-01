use crate::log::Logger;
use crate::runtime::Runtime;
use anyhow::Result;
use std::path::PathBuf;

mod log;
mod runtime;

struct ProtoCore {
    logger: Logger,
}

impl ProtoCore {
    fn new(wasm_filename: &str) -> Result<ProtoCore> {
        let mut log_dir = PathBuf::from("/tmp/ves_proto");
        std::fs::create_dir_all(&log_dir)?;
        log_dir.push(wasm_filename);
        let logger = Logger::from_file(log_dir)?;
        Ok(Self { logger })
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let wasm_file = PathBuf::from(&args[1]).canonicalize().unwrap();
    println!("Running core.");
    println!(
        "Loading WASM file: {}",
        wasm_file.as_path().to_str().unwrap()
    );

    let wasm_filename = wasm_file.file_name().unwrap().to_str().unwrap();
    let core = ProtoCore::new(wasm_filename).unwrap();
    let mut runtime = Runtime::from_path(wasm_file.as_path(), core).unwrap();
    println!("Creating game instance.");
    let instance_ptr = runtime.create_instance().unwrap();

    println!("Starting game loop.");
    // TODO: Implement actual game loop with SDL
    runtime.step(instance_ptr).unwrap();
    runtime.step(instance_ptr).unwrap();
    runtime.step(instance_ptr).unwrap();
    runtime.step(instance_ptr).unwrap();
    runtime.step(instance_ptr).unwrap();
}
