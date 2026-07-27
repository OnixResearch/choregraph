use std::env;
use std::process::ExitCode;

use choregraph_cli::{Command, parse_command, render_foundation_report};

fn main() -> ExitCode {
    let arguments: Vec<String> = env::args().skip(1).collect();

    match parse_command(&arguments) {
        Ok(Command::Boundary) => {
            print!("{}", render_foundation_report());
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}
