// This file contains the "check" subcommand

use crate::dudect::{DudeCT, DudeResult};
use crate::errors::SideFuzzError;
use crate::util::*;
use crate::wasm::WasmModule;

pub struct Check {
  module: WasmModule,
  input: InputPair,
}

impl Check {

  // Create a new check command with the given wasm module and two inputs
  pub fn new(module: WasmModule, first: Vec<u8>, second: Vec<u8>) -> Result<Self, SideFuzzError> {
    if first.len() != second.len() {
      return Err(SideFuzzError::InputsDifferentSizes);
    }

    if first.len() != module.fuzz_len() {
      return Err(SideFuzzError::InputsWrongSize(module.fuzz_len()));
    }

    Ok(Check {
      module: module,
      input: InputPair { first, second },
    })
  }

  pub fn from_file(filename: &str, first: Vec<u8>, second: Vec<u8>) -> Result<Self, SideFuzzError> {
    let module = WasmModule::from_file(filename)?;
    Self::new(module, first, second)
  }

  pub fn run(&mut self) {
    // Get the instruction counts
    let scored_input = ScoredInputPair::generate(
      &mut self.module,
      self.input.first.to_vec(),
      self.input.second.to_vec(),
    );

    // Construct DudeCT
    // Return success on t = 4.5 (very high confidence)
    // Give up on t < 0.674 (50% confidence) when over 1 million samples.
    let mut dudect = DudeCT::new(
      4.5,     // Success t-value
      0.674,   // Give up t-value
      100_000, // Give up min samples
      &self.input.first,
      &self.input.second,
      self.module.clone(),
    );

    loop {
      let (t, result) = dudect.sample(10_000);
      let p = p_value_from_t_value(t);

      println!(
        "samples: {}, t-value: {}, confidence: {}%",
        dudect.len(),
        t,
        (1.0 - p) * 100.0
      );

      match result {
        DudeResult::Ok => {
          println!(
                "Found timing difference of {} instructions between these two inputs with {}% confidence:\ninput 1: {} ({} instructions) \ninput 2: {} ({} instructions)",
                scored_input.score,
                (1.0 - p) * 100.0,
                hex::encode(&scored_input.pair.first),
                scored_input.highest,
                hex::encode(&scored_input.pair.second),
                scored_input.lowest,
              );
          std::process::exit(0);
        }
        DudeResult::Err => {
          println!(
                "Candidate input pair rejected: t-statistic small after many samples. Target is probably constant time."
              );
          std::process::exit(0);
        }
        DudeResult::Progress => {
          continue;
        }
      }
    }
  }
}