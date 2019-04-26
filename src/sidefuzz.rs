use crate::dudect::{DudeCT, DudeResult};
use crate::optimizer::Optimizer;
use std::convert::TryFrom;

use wasmi::{ImportsBuilder, Module, ModuleInstance, NopExternals, RuntimeValue};

pub struct SideFuzz {
  wasm_module: Module,
}

impl SideFuzz {
  pub fn new(wasm_module: Module) -> Self {
    SideFuzz { wasm_module }
  }

  pub fn run(&self) {

    // Instantiate a module with empty imports and
    // assert that there is no `start` function.
    let module_instance = ModuleInstance::new(&self.wasm_module, &ImportsBuilder::default())
      .expect("failed to instantiate wasm module")
      .assert_no_start();

    // Call the "len" exported function to get the desired fuzzing length.
    let fuzz_len = module_instance
      .invoke_export("len", &[], &mut NopExternals)
      .expect("wasm module did not export a len() function")
      .unwrap();

    let fuzz_len = match fuzz_len {
      wasmi::RuntimeValue::I32(inner) => inner as usize,
      _ => {
        // TODO: return don't panic;
        panic!("Invalid len() return type");
      }
    };

    // Get memory instance exported by name 'mem' from the module instance.
    let internal_mem = module_instance.export_by_name("memory");
    let internal_mem = internal_mem.expect("Module expected to have 'mem' export");
    let internal_mem = internal_mem.as_memory().unwrap();

    let mut optimizer = Optimizer::new(fuzz_len, |first: &[u8], second: &[u8]| {

      // First
      wasmi::reset_instruction_count();
      internal_mem.set(0, first).unwrap();
      let result = module_instance.invoke_export(
        "sidefuzz",
        &[
          RuntimeValue::I32(0),
          RuntimeValue::I32(i32::try_from(first.len()).unwrap()),
        ],
        &mut NopExternals,
      );
      if result.is_err() {
        return (std::f64::NEG_INFINITY, 0.0, 0.0); // wasm trapped, fitness set to negative-infinity.
      }
      let first_instructions = wasmi::get_instruction_count();

      // Second
      wasmi::reset_instruction_count();
      internal_mem.set(0, second).unwrap();
      let result = module_instance.invoke_export(
        "sidefuzz",
        &[
          RuntimeValue::I32(0),
          RuntimeValue::I32(i32::try_from(second.len()).unwrap()),
        ],
        &mut NopExternals,
      );
      if result.is_err() {
        return (std::f64::NEG_INFINITY, 0.0, 0.0); // wasm trapped, fitness set to negative-infinity.
      }
      let second_instructions = wasmi::get_instruction_count();

      // Difference
      let highest;
      let lowest;
      if first_instructions >= second_instructions {
        highest = first_instructions;
        lowest = second_instructions;
      } else {
        highest = second_instructions;
        lowest = first_instructions;
      }
      let diff = highest - lowest;

      (diff as f64, highest as f64, lowest as f64)
    });

    println!("Evolving candidate input pairs");
    let mut best = crate::ScoredInputPair::default(); // defaults to score of zero.
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
            &self.wasm_module,
          );

          loop {
            let (t, result) = dudect.sample(10_000);
            let p = crate::p_value_from_t_value(t);

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
                best = crate::ScoredInputPair::default();
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