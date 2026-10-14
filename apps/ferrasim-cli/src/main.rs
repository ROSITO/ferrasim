//! Informational entry point for the FerraSim workspace.

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    match args.as_slice() {
        [] => {
            print_help();
            ExitCode::SUCCESS
        }
        [flag] if flag == "--help" || flag == "-h" => {
            print_help();
            ExitCode::SUCCESS
        }
        [flag] if flag == "--version" || flag == "-V" => {
            println!("ferrasim {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("Unsupported command: simulation commands are not implemented yet.");
            ExitCode::from(2)
        }
    }
}

fn print_help() {
    println!(
        "FerraSim — Rust simulation platform\n\
         Usage: ferrasim [--help | --version]\n\
         Sprint S00: workspace foundations only.\n\
         No model validation or simulation is implemented yet."
    );
}
