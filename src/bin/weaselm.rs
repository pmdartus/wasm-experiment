use std::env;
use std::fs;
use std::io;

// TODO: the package structure for the export
use weaselm::decoder::modules::{decode};
use weaselm::validation::modules::{validate};

// TODO: Understand how this io package work!
fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let filename = match args.get(1) {
        Some(p) => p,
        _ => panic!("No file argument found"),
    };

    let file = fs::read(filename)?;

    let module = decode(&file[..]).unwrap();
    validate(&module).unwrap();

    println!("{:#?}", module);

    Ok(())
}
