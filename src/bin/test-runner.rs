use std::env;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use serde_json;

use weaselm::decoder;

mod manifest {
    use serde::Deserialize;

    #[derive(Deserialize, Debug)]
    pub struct Manifest {
        pub source_filename: String,
        pub commands: Vec<Command>,
    }

    #[derive(Deserialize, Debug)]
    #[serde(tag = "type")]
    pub enum Command {
        #[serde(rename = "action")]
        Action {
            line: u32,
            action: Action,
            expected: Vec<Value>,
        },

        #[serde(rename = "assert_exhaustion")]
        AssertExhaustion {
            line: u32,
            action: Action,
            text: String,
            expected: Vec<Value>,
        },

        #[serde(rename = "assert_invalid")]
        AssertInvalid {
            line: u32,
            filename: String,
            text: String,
        },

        #[serde(rename = "assert_malformed")]
        AssertMalformed {
            line: u32,
            filename: String,
            text: String,
        },

        #[serde(rename = "assert_trap")]
        AssertTrap {
            line: u32,
            action: Action,
            text: String,
            expected: Vec<Value>,
        },

        #[serde(rename = "assert_uninstantiable")]
        AssertUninstantiable {
            line: u32,
            filename: Option<String>,
            text: String,
        },

        #[serde(rename = "assert_unlinkable")]
        AssertUnlinkable {
            line: u32,
            filename: Option<String>,
            text: String,
        },

        #[serde(rename = "assert_return")]
        AssertReturn {
            line: u32,
            action: Action,
            expected: Vec<Value>,
        },

        #[serde(rename = "assert_return_arithmetic_nan")]
        AssertReturnArithmeticNan {
            line: u32,
            action: Action,
            expected: Vec<Value>,
        },

        #[serde(rename = "assert_return_canonical_nan")]
        AssertReturnCanonicalNan {
            line: u32,
            action: Action,
            expected: Vec<Value>,
        },

        #[serde(rename = "module")]
        Module { line: u32, filename: String },

        #[serde(rename = "register")]
        Register {
            line: u32,
            name: Option<String>,
            #[serde(alias = "as")]
            alias: String,
        },
    }

    #[derive(Deserialize, Debug)]
    #[serde(tag = "type")]
    pub enum Action {
        #[serde(rename = "invoke")]
        Invoke { field: String, args: Vec<Value> },

        #[serde(rename = "get")]
        Get {
            field: String,
            module: Option<String>,
        },
    }

    #[derive(Deserialize, Debug)]
    #[serde(tag = "type")]
    pub enum Value {
        #[serde(rename = "i32")]
        I32 { value: Option<String> },
        #[serde(rename = "i64")]
        I64 { value: Option<String> },
        #[serde(rename = "f32")]
        F32 { value: Option<String> },
        #[serde(rename = "f64")]
        F64 { value: Option<String> },
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let dir = args.get(1).expect("No test directory found");

    let test_dir = Path::new(dir);
    let manifest_files: Vec<_> = fs::read_dir(test_dir)
        .unwrap()
        .map(|f| f.unwrap().path())
        .filter(|f| f.extension().unwrap() == "json")
        .collect();

    for manifest_file in manifest_files {
        let reader = BufReader::new(File::open(manifest_file).unwrap());
        let manifest: manifest::Manifest = serde_json::from_reader(reader).unwrap();

        run_manifest(&test_dir, manifest);
    }
}

fn run_manifest(test_dir: &Path, manifest: manifest::Manifest) {
    let filename = Path::new(&manifest.source_filename)
        .file_name()
        .unwrap()
        .to_str()
        .unwrap();
    println!("Running test for: {:}", filename);

    for command in manifest.commands {
        match command {
            manifest::Command::Module { line: _, filename } => {
                let module_path = test_dir.join(filename);
                let module_path_string  = module_path.to_str().unwrap();

                let file = fs::read(module_path_string).unwrap();
                let res = decoder::decode(&file[..]);

                match res {
                    Err(err) => println!("❌ FAILED: {:?} {}", module_path_string, err),
                    _ => println!("✔️ PASS {:?}", module_path_string),
                }
            }
            _ => (),
        }
    }
}
