use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use colored::*;

mod manifest;
use manifest::{Command, CommandAssertMalformed, CommandModule, Manifest};

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

    fn ignore(test_name: String, file_name: String, line: u32) -> TestResult {
        TestResult {
            test_name,
            line,
            file_name,
            state: TestState::Ignore,
        }
    }
}

#[derive(Debug, PartialEq)]
enum TestState {
    Pass,
    Fail { message: String },
    Ignore,
}

pub fn run(config: &RunnerConfig) -> i32 {
    let manifests = get_manifests(config);

    let report: Vec<(Manifest, Vec<TestResult>)> = manifests
        .into_iter()
        .map(|manifest| {
            let results = run_suite(&manifest, config);
            (manifest, results)
        })
        .collect();

    print_report(&report);

    report
        .iter()
        .flat_map(|(_, results)| results)
        .filter(|r| match &r.state {
            TestState::Fail { message: _message } => true,
            _ => false,
        })
        .count() as i32
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

    manifests.sort_by(|a, b| a.source_filename.cmp(&b.source_filename));

    manifests
}

fn run_suite(manifest: &Manifest, config: &RunnerConfig) -> Vec<TestResult> {
    println!("{}", manifest.source_filename.bold());

    manifest
        .commands
        .iter()
        .enumerate()
        .map(|(index, command)| {
            let result: TestResult = match command {
                Command::Module(command) => test_module_instantiation(command, index, config),
                Command::AssertMalformed(command) => test_module_malformed(command, index, config),

                Command::Action(command) => {
                    TestResult::ignore(format!("#{} Action", index), String::from(""), command.line)
                }

                Command::AssertExhaustion(command) => TestResult::ignore(
                    format!("#{} Exhaustion: {}", index, command.text),
                    String::from(""),
                    command.line,
                ),

                Command::AssertInvalid(command) => TestResult::ignore(
                    format!("#{} Invalid module: {}", index, command.text),
                    command.filename.clone(),
                    command.line,
                ),

                Command::AssertTrap(command) => TestResult::ignore(
                    format!("#{} Trap: {}", index, command.text),
                    String::from(""),
                    command.line,
                ),

                Command::AssertUninstantiable(command) => TestResult::ignore(
                    format!("#{} Uninstantiable  module: {}", index, command.text),
                    command.filename.clone().unwrap_or_else(|| String::from("")),
                    command.line,
                ),

                Command::AssertUnlinkable(command) => TestResult::ignore(
                    format!("#{} Unlinkable module: {}", index, command.text),
                    command.filename.clone().unwrap_or_else(|| String::from("")),
                    command.line,
                ),

                Command::AssertReturn(command) => {
                    TestResult::ignore(format!("#{} Return", index), String::from(""), command.line)
                }

                Command::AssertReturnArithmeticNan(command) => TestResult::ignore(
                    format!("#{} Return arithmetic NaN", index),
                    String::from(""),
                    command.line,
                ),

                Command::AssertReturnCanonicalNan(command) => TestResult::ignore(
                    format!("#{} Return canonical NaN", index),
                    String::from(""),
                    command.line,
                ),

                Command::Register(command) => TestResult::ignore(
                    format!("#{} Register", index),
                    String::from(""),
                    command.line,
                ),
            };

            match &result.state {
                TestState::Pass => {
                    println!("  {}", result.test_name.green());
                }
                TestState::Ignore => {
                    println!("  {}", result.test_name.cyan());
                }
                TestState::Fail { message } => {
                    println!("  {}", result.test_name.red());
                    println!("    {}", message.bright_black());
                }
            };

            result
        })
        .collect()
}

fn test_module_instantiation(
    command: &CommandModule,
    index: usize,
    config: &RunnerConfig,
) -> TestResult {
    let test_name = format!("#{} Instantiate module", index);

    let module_path = Path::new(&config.dirname)
        .join(&command.filename)
        .into_os_string();
    let file = fs::read(module_path).unwrap();

    match decode(&file[..]) {
        Err(err) => {
            let message = format!(
                "Expected module to instantiate but received error: {} (offset: {}, file: {})",
                err.message, err.offset, command.filename
            );
            TestResult::fail(
                test_name,
                command.filename.to_string(),
                command.line,
                message,
            )
        }
        _ => TestResult::pass(test_name, command.filename.to_string(), command.line),
    }
}

fn test_module_malformed(
    command: &CommandAssertMalformed,
    index: usize,
    config: &RunnerConfig,
) -> TestResult {
    let test_name = format!("#{} Malformed module: {}", index, command.text);

    let module_path = Path::new(&config.dirname)
        .join(&command.filename)
        .into_os_string();
    let file = fs::read(module_path).unwrap();

    match decode(&file[..]) {
        Ok(_) => {
            let message = format!(
                "Expected module to be malformed but parsed properly (file: {})",
                command.filename
            );
            TestResult::fail(
                test_name,
                command.filename.to_string(),
                command.line,
                message,
            )
        }
        _ => TestResult::pass(test_name, command.filename.to_string(), command.line),
    }
}

fn print_report(report: &Vec<(Manifest, Vec<TestResult>)>) {
    let results: Vec<&TestResult> = report.iter().flat_map(|(_, results)| results).collect();

    let passing_count = results
        .iter()
        .filter(|r| r.state == TestState::Pass)
        .count();
    let failing_count = results
        .iter()
        .filter(|r| match &r.state {
            TestState::Fail { message: _message } => true,
            _ => false,
        })
        .count();
    let ignored_count = results
        .iter()
        .filter(|r| r.state == TestState::Ignore)
        .count();

    println!();
    println!(
        "    {}",
        format!("{} passing", passing_count).bold().green()
    );
    println!("    {}", format!("{} failing", failing_count).bold().red());
    println!("    {}", format!("{} ingored", ignored_count).bold().cyan());
    println!();

    for (manifest, results) in report {
        let message = results
            .iter()
            .fold(String::new(), |acc, result| match &result.state {
                TestState::Fail { message } => format!(
                    "{}\n  {}\n    {}\n",
                    acc,
                    result.test_name.red(),
                    message.bright_black()
                ),
                _ => acc,
            });

        if !message.is_empty() {
            println!("{}", manifest.source_filename.bold());
            println!("{}", message);
        }
    }
}
