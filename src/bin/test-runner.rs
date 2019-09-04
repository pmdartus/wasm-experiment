use std::env;
use std::fs;
use std::io::BufReader;
use std::path::Path;
use std::fs::File;

use serde_json::{Value};

fn main() {
    let args: Vec<String> = env::args().collect();
    let dir = match args.get(1) {
        Some(p) => p,
        _ => panic!("No test directory found"),
    };

    let test_dir = Path::new(dir);
    let test_files: Vec<_> = fs::read_dir(test_dir)
        .unwrap()
        .map(|f| f.unwrap().path())
        .filter(|f| f.extension().unwrap() == "json")
        .collect();

    for test_file in test_files {
        println!("{:?}", test_file);
        
        let reader = BufReader::new(File::open(test_file).unwrap());
        let data: Value = serde_json::from_reader(reader).unwrap();

        let tests = match &data["commands"] {
            Value::Array(x) => x,
            _ => unreachable!()
        };

        for test in tests {
            println!("type {:?}", test["type"])
        }
    }
}