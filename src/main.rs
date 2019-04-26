use clap::{App, Arg, SubCommand};
use failure::Error;
use sidefuzz::sidefuzz::SideFuzz;
use std::fs::File;
use std::io::prelude::*;
use wasmi::Module;

fn load_from_file(filename: &str) -> Module {
    let mut file = File::open(filename).unwrap();
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();
    Module::from_buffer(buf).unwrap()
}

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
        );

    let matches = app.clone().get_matches();

    // Fuzz command
    if let Some(sub_match) = matches.subcommand_matches("fuzz") {
        let filename = sub_match.value_of("wasm-file").unwrap();
        let module = load_from_file(filename);
        let fuzzer = SideFuzz::new(module);
        fuzzer.run();
    } else {
        app.print_long_help()?;
    }

    Ok(())
}