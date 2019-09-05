use std::env;
use std::fs;
use std::io;

use weaselm::decoder;

// TODO: Understand how this io package work!
fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let filename = match args.get(1) {
        Some(p) => p,
        _ => panic!("No file argument found"),
    };

    let file = fs::read(filename)?;
    let module = decoder::decode(&file[..]);

    println!("{:#?}", module);

    Ok(())
}
