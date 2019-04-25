use std::env::args;
use std::fs::File;
use wasmi::Module;
use sidefuzz::SideFuzz;

fn load_from_file(filename: &str) -> Module {
    use std::io::prelude::*;
    let mut file = File::open(filename).unwrap();
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();
    Module::from_buffer(buf).unwrap()
}

fn main() {
    let args: Vec<_> = args().collect();
    if args.len() != 3 {
        println!("Usage: {} <wasm file> <input-length>", args[0]);
        return;
    }

    // Here we load module using dedicated for this purpose
    // `load_from_file` function (which works only with modules)
    let module = load_from_file(&args[1]);

    let fuzzer = SideFuzz::new(args[2].parse().unwrap(), module);

    fuzzer.run();
}