// This file contains the "count" subcommand

use crate::errors::SideFuzzError;
use crate::wasm::WasmModule;

pub struct Count {
  module: WasmModule,
  input: Vec<u8>,
}

impl Count {

  // Create a new check command with the given wasm module and two inputs
  pub fn new(module: WasmModule, input: Vec<u8>) -> Result<Self, SideFuzzError> {
    if input.len() != module.fuzz_len() {
      return Err(SideFuzzError::InputsWrongSize(module.fuzz_len()));
    }

    Ok(Count {
      module: module,
      input: input,
    })
  }

  pub fn from_file(filename: &str, input: Vec<u8>) -> Result<Self, SideFuzzError> {
    let module = WasmModule::from_file(filename)?;
    Self::new(module, input)
  }

  pub fn run(&mut self) {
    let num_instructions = self.module.count_instructions(&self.input);
    match num_instructions {
      Ok(num) => {
        println!("{}", num);
        std::process::exit(0);
      }
      Err(e) => {
        println!("{}", e);
        std::process::exit(1);
      }
    }
  }
}