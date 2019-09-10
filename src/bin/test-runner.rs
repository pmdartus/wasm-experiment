use std::env;

use weaselm::test_runner::{run, RunnerConfig};

fn main() {
    let args: Vec<String> = env::args().collect();
    let dirname = args.get(1).expect("No test directory found");

    run(&RunnerConfig {
        dirname: dirname.clone()
    });
}
