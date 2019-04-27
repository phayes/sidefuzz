// This file contains the "fuzz" subcommand

use crate::dudect::{DudeCT, DudeResult};
use crate::optimizer::Optimizer;
use crate::util::*;
use crate::errors::SideFuzzError;
use crate::wasm::WasmModule;

pub struct Fuzz {
  module: WasmModule,
}

impl Fuzz {
  pub fn new(module: WasmModule) -> Self {
    Fuzz { module }
  }

  pub fn from_file(filename: &str) -> Result<Self, SideFuzzError> {
    let module = WasmModule::from_file(filename)?;
    Ok(Self::new(module))
  }

  pub fn run(&mut self) {
    // Grab a copy of module bytes, we will pass this into DudeCT later
    let module_bytes = self.module.bytes();

    // Print approximately fuzzing duration
    // duration = run-time * aprox-num-loops * num-generations-per-loop * population-size
    let duration = self.module.measure_time() * 40.0 * 500.0 * 1000.0;
    println!("Fuzzing will take approximately {:.*}", 0, duration);

    let mut optimizer = Optimizer::new(self.module.fuzz_len(), |first: &[u8], second: &[u8]| {
      ScoredInputPair::generate(&mut self.module, first.to_vec(), second.to_vec())
    });

    println!("Evolving candidate input pairs");
    let mut best = ScoredInputPair::default(); // defaults to score of zero.
    let mut moving_window = vec![0.0; 10]; // Moving window of size 10
    loop {
      // Check results once every 500 genearations
      for _ in 0..500 {
        optimizer.step();
      }
      let population = optimizer.scored_population();
      let pop_best = population[0].clone(); // Best of this population is ordered first.

      if pop_best.score != 0.0 {
        println!(
          "{} {} {}",
          pop_best.score,
          hex::encode(&population[0].pair.first),
          hex::encode(&population[0].pair.second)
        );
      } else {
        println!("Looks constant-time so far...");
      }

      // Adjust moving window
      moving_window.remove(0);
      moving_window.push(pop_best.score);

      if pop_best.score > best.score {
        best = pop_best;
      }

      if best.score != 0.0 {
        // Check the moving window is entirely the same as the best, this means we're maxed out.
        let mut local_optimum = true;
        for score in moving_window.iter() {
          if score != &best.score {
            local_optimum = false;
            break;
          }
        }

        if local_optimum {
          println!(
            "Checking {} {}",
            hex::encode(&best.pair.first),
            hex::encode(&best.pair.second)
          );

          // Construct DudeCT
          // Return success on t = 4.5 (very high confidence)
          // Give up on t < 0.674 (50% confidence) when over 1 million samples.
          let mut dudect = DudeCT::new(
            4.5,     // Success t-value
            0.674,   // Give up t-value
            100_000, // Give up min samples
            &best.pair.first,
            &best.pair.second,
            WasmModule::new(module_bytes.clone()),
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
                "Found timing difference of {} instructions between these two inputs with {}% confidence:\ninput 1: {}\ninput 2: {}",
                best.score,
                (1.0 - p) * 100.0,
                hex::encode(&best.pair.first),
                hex::encode(&best.pair.second)
              );
                std::process::exit(0);
              }
              DudeResult::Err => {
                best = ScoredInputPair::default();
                println!(
                "Candidate input pair rejected: t-statistic small after many samples. Continuing to evolve candidate inputs."
              );
                break;
              }
              DudeResult::Progress => {
                continue;
              }
            }
          }
        }
      }
    }
  }

}
