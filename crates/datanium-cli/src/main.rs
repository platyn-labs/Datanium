use std::{env, process::ExitCode};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();

    match args.as_slice() {
        [] => {
            print_help();
            ExitCode::SUCCESS
        }
        [arg] if arg == "--help" || arg == "-h" => {
            print_help();
            ExitCode::SUCCESS
        }
        [arg] if arg == "--version" || arg == "-v" => {
            println!("datanium {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("Unknown arguments. Use datanium --help.");
            ExitCode::from(2)
        }
    }
}

fn print_help() {
    println!(
        "Datanium CLI
Verwendung:
    datanium [Option]

Option:
    -h, --help     show help
    -v, --version  show version"
    )
}
