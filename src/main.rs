use clap::{App, Arg, SubCommand};
use failure::Error;
use sidefuzz::fuzz::Fuzz;

fn main() -> Result<(), Error> {
    color_backtrace::install();

    let mut app = App::new("sidefuzz")
        .version("0.1.0")
        .author("Patrick Hayes <patrick.d.hayes@gmail.com>")
        .about("Fuzzes for timing side-channel vulnerabilities using wasm")
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

        fuzz.run();
    } else {
        app.print_long_help()?;
    }

    Ok(())
}