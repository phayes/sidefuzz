use clap::{App, Arg, SubCommand};
use failure::Error;


use sidefuzz::check::Check;
use sidefuzz::count::Count;
use sidefuzz::fuzz::Fuzz;
fn main() -> Result<(), Error> {
    color_backtrace::install();

    let mut app = App::new("sidefuzz")
        .version("0.1.0")
        .author("Patrick Hayes <patrick.d.hayes@gmail.com>")
        .about("Fuzzes for timing side-channel vulnerabilities using wasm. \nhttps://github.com/phayes/sidefuzz")
        .arg(
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity"),
        )
        .subcommand(
            SubCommand::with_name("fuzz")
                .about("fuzzes wasm file, generating variable-time input pairs")
                .arg(
                    Arg::with_name("wasm-file")
                        .help("wasm file fuzzing target")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("check")
                .about("Check wasm file with two inputs")
                .arg(
                    Arg::with_name("wasm-file")
                        .help("wasm file fuzzing target")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("input-1")
                        .help("first input in hexedecimal format")
                        .required(true)
                        .index(2),
                )
                .arg(
                    Arg::with_name("input-2")
                        .help("second input in hexedecimal format")
                        .required(true)
                        .index(3),
                ),
        )
        .subcommand(
            SubCommand::with_name("count")
                .about("Count the number of instructions executed for a single input.")
                .arg(
                    Arg::with_name("wasm-file")
                        .help("wasm file fuzzing target")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("input")
                        .help("Input in hexedecimal format")
                        .required(true)
                        .index(2),
                )
        );

    let matches = app.clone().get_matches();

    // Fuzz command
    if let Some(sub_match) = matches.subcommand_matches("fuzz") {
        let filename = sub_match.value_of("wasm-file").unwrap();
        let mut fuzz = match Fuzz::from_file(filename) {
            Ok(fuzz) => fuzz,
            Err(err) => {
                println!("Error: {}", err);
                std::process::exit(1);
            }
        };

        let result = fuzz.run();
        match result {
            Ok(_) => std::process::exit(0),
            Err(err) => {
                println!("Error: {}", err);
                std::process::exit(0);
            }
        }
    }

    // Check command
    if let Some(sub_match) = matches.subcommand_matches("check") {
        let filename = sub_match.value_of("wasm-file").unwrap();

        let first = sub_match.value_of("input-1").unwrap();
        let first = hex::decode(first)?;

        let second = sub_match.value_of("input-2").unwrap();
        let second = hex::decode(second)?;

        let mut check = match Check::from_file(filename, first, second) {
            Ok(check) => check,
            Err(err) => {
                println!("Error: {}", err);
                std::process::exit(1);
            }
        };

        let result = check.run();
        match result {
            Ok(_) => std::process::exit(0),
            Err(err) => {
                println!("Error: {}", err);
                std::process::exit(0);
            }
        }
    }

    // Count command
    if let Some(sub_match) = matches.subcommand_matches("count") {
        let filename = sub_match.value_of("wasm-file").unwrap();

        let input = sub_match.value_of("input").unwrap();
        let input = hex::decode(input)?;

        let mut count = match Count::from_file(filename, input) {
            Ok(count) => count,
            Err(err) => {
                println!("Error: {}", err);
                std::process::exit(1);
            }
        };

        count.run();
        std::process::exit(0);
    }

    app.print_long_help()?;
    Ok(())
}