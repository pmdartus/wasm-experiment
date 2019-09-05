use std::env;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use serde::Deserialize;
use serde_json;

#[derive(Deserialize, Debug)]
struct TestManifest {
    source_filename: String,
    commands: Vec<TestCommand>,
}

#[derive(Deserialize, Debug)]
struct TestCommand {
    #[serde(alias = "type")]
    command_type: String,
    line: u32,
    name: Option<String>,
    filename: Option<String>,
    action: Option<TestAction>,
    text: Option<String>,
    expected: Option<Vec<Value>>,
}

#[derive(Deserialize, Debug)]
struct TestAction {
    #[serde(alias = "type")]
    action_type: String,
    field: String,
    module: Option<String>,
    args: Option<Vec<Value>>,
}

#[derive(Deserialize, Debug)]
struct Value {
    #[serde(alias = "type")]
    value_type: String,
    value: Option<String>,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let dir = match args.get(1) {
        Some(p) => p,
        _ => panic!("No test directory found"),
    };

    let test_dir = Path::new(dir);
    let manifest_files: Vec<_> = fs::read_dir(test_dir)
        .unwrap()
        .map(|f| f.unwrap().path())
        .filter(|f| f.extension().unwrap() == "json")
        .collect();

    for manifest_file in manifest_files {
        println!("{:?}", manifest_file);

        let reader = BufReader::new(File::open(manifest_file).unwrap());
        let manifest: TestManifest = serde_json::from_reader(reader).unwrap();

        println!("{:#?}", manifest);
    }
}
