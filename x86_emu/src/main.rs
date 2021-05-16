use std::env;
use std::fs::File;
use file_loader::LoadableFile;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let mut file = File::open(filename)?;
    let loader = file.get_loader().unwrap();
    loader.load(&mut file);
    Ok(())
}
