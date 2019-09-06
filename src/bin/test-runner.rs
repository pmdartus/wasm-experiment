use std::env;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use colored::*;
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

#[derive(Debug)]
struct TestResult {
    test_name: String,
    file_name: String,
    line: u32,
    state: TestState,
}

#[derive(Debug)]
enum TestState {
    Pass,
    Fail { message: String },
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let dir = args.get(1).expect("No test directory found");

    let filter = if args.len() > 2 {
        Some(args.get(2).unwrap())
    } else {
        None
    };

    let test_dir = Path::new(dir);
    let manifest_files: Vec<_> = fs::read_dir(test_dir)
        .unwrap()
        .map(|f| f.unwrap().path())
        .filter(|f| {
            if f.extension().unwrap() != "json" {
                return false;
            }

            match filter {
                Some(filename) => f.file_name().unwrap().to_str().unwrap() == filename,
                None => true,
            }
        })
        .collect();

    for manifest_file in manifest_files {
        let reader = BufReader::new(File::open(manifest_file).unwrap());
        let manifest: manifest::Manifest = serde_json::from_reader(reader).unwrap();

        println!("{}", manifest.source_filename.bold());

        let results = run_manifest(&test_dir, manifest);

        for result in results {
            match result.state {
                TestState::Pass => {
                    println!("  {}", result.test_name.bright_green());
                }
                TestState::Fail { message } => {
                    println!("  {}", result.test_name.red());
                    println!("    {}", message.bright_black());
                }
            }
        }

        println!("")
    }
}

fn run_manifest(test_dir: &Path, manifest: manifest::Manifest) -> Vec<TestResult> {
    let mut command_index = 0;

    manifest
        .commands
        .iter()
        .map(move |command| {
            command_index += 1;

            match command {
                manifest::Command::Module { line, filename } => {
                    let test_name = format!("#{} Instantiate module", command_index);

                    let module_path = test_dir.join(filename);
                    let module_path_string = module_path.to_str().unwrap();

                    let file = fs::read(module_path_string).unwrap();
                    let res = decoder::decode(&file[..]);

                    match res {
                        Err(err) => Some(TestResult {
                            test_name,
                            line: *line,
                            file_name: String::from(filename),
                            state: TestState::Fail {
                                message: format!(
                                    "{} (offset: {}, file: {})",
                                    err.message, err.offset, module_path_string
                                ),
                            },
                        }),
                        _ => Some(TestResult {
                            test_name,
                            line: *line,
                            file_name: String::from(filename),
                            state: TestState::Pass,
                        }),
                    }
                }
                // manifest::Command::AssertMalformed { line, filename, text } => {
                //     let module_path = test_dir.join(filename);
                //     let module_path_string  = module_path.to_str().unwrap();

                //     let file = fs::read(module_path_string).unwrap();
                //     let res = decoder::decode(&file[..]);

                //     match res {
                //         Err(_err) => println!("✅ PASS {:?}", module_path_string),
                //         Ok(_module) => {
                //             // println!("{:#?}", _module);
                //             println!("❌ FAILED: {:?} (line: {:}) {}", module_path_string, line, text)
                //         },
                //     }
                // }
                _ => None,
            }
        })
        .filter_map(|result| result)
        .collect()
}
