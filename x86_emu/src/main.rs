use file_loader::LoadableFile;
use processor::ProcessableMemoryMap;
use std::env;
use std::fs::File;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let mut file = File::open(filename)?;
    let loader = file.loader().unwrap();
    let mut memory_map = loader.load(&mut file);
    let mut processor = memory_map.processor_impl().unwrap();
    processor.init(&mut *memory_map);
    while processor.running() {
        processor.tick(&mut *memory_map);
    }
    Ok(())
}
