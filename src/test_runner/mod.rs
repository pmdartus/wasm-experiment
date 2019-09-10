use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use colored::*;

mod manifest;
use manifest::{Command, Manifest};

use crate::decoder::modules::decode;

pub struct RunnerConfig {
    pub dirname: String,
}

#[derive(Debug)]
struct TestResult {
    test_name: String,
    file_name: String,
    line: u32,
    state: TestState,
}

impl TestResult {
    fn pass(test_name: String, file_name: String, line: u32) -> TestResult {
        TestResult {
            test_name,
            line,
            file_name,
            state: TestState::Pass,
        }
    }

    fn fail(test_name: String, file_name: String, line: u32, message: String) -> TestResult {
        TestResult {
            test_name,
            line,
            file_name,
            state: TestState::Fail { message },
        }
    }
}

#[derive(Debug)]
enum TestState {
    Pass,
    Fail { message: String },
}

pub fn run(config: &RunnerConfig) {
    let manifests = get_manifests(config);
    for manifest in &manifests {
        run_suite(manifest, config);
    }
}

fn get_manifests(config: &RunnerConfig) -> Vec<Manifest> {
    let mut manifests: Vec<Manifest> = fs::read_dir(config.dirname.clone())
        .unwrap()
        .map(|f| f.unwrap().path())
        .filter(|f| f.extension().unwrap() == "json")
        .map(|f| {
            let reader = BufReader::new(File::open(f).unwrap());
            let manifest: Manifest = serde_json::from_reader(reader).unwrap();
            manifest
        })
        .collect();

    manifests.sort_by(|a, b| {
        a.source_filename.cmp(&b.source_filename)
    });

    manifests
}

fn run_suite(manifest: &Manifest, config: &RunnerConfig) {
    println!("{}", manifest.source_filename.bold());

    let mut index = 0;

    for command in &manifest.commands {
        let result = match command {
            Command::Module { line, filename } => {
                Some(test_module_instantiation(filename, *line, config))
            }
            Command::AssertMalformed {
                line,
                filename,
                text,
            } => Some(test_module_malformed(filename, *line, text, config)),
            _ => None,
        };

        if let Some(test_result) = result {
            match test_result.state {
                TestState::Pass => {
                    println!("  {} {}", format!("#{}", index).green(), test_result.test_name.green());
                }
                TestState::Fail { message } => {
                    println!("  {} {}", format!("#{}", index).red(), test_result.test_name.red());
                    println!("    {}", message.bright_black());
                }
            };
        }

        index += 1;
    }
}

fn test_module_instantiation(file_name: &String, line: u32, config: &RunnerConfig) -> TestResult {
    let test_name = format!("Instantiate module");

    let module_path = Path::new(&config.dirname).join(file_name).into_os_string();
    let file = fs::read(module_path).unwrap();

    match decode(&file[..]) {
        Err(err) => {
            let message = format!(
                "Expected module to instantiate but received error: {} (offset: {}, file: {})",
                err.message, err.offset, file_name
            );
            TestResult::fail(test_name, file_name.to_string(), line, message)
        }
        _ => TestResult::pass(test_name, file_name.to_string(), line),
    }
}

fn test_module_malformed(
    file_name: &String,
    line: u32,
    text: &String,
    config: &RunnerConfig,
) -> TestResult {
    let test_name = format!("Malformed module: {}", text);

    let module_path = Path::new(&config.dirname).join(file_name).into_os_string();
    let file = fs::read(module_path).unwrap();

    match decode(&file[..]) {
        Ok(_) => {
            let message = format!(
                "Expected module to be malformed but parsed properly (file: {})",
                file_name
            );
            TestResult::fail(test_name, file_name.to_string(), line, message)
        }
        _ => TestResult::pass(test_name, file_name.to_string(), line),
    }
}
