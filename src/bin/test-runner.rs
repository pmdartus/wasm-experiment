use std::{env, process};

use weaselm::test_runner::{run, RunnerConfig};

fn main() {
    let args: Vec<String> = env::args().collect();
    let dirname = args.get(1).expect("No test directory found");

    let exit_code = run(&RunnerConfig {
        dirname: dirname.clone(),
    });

    process::exit(exit_code);
}
