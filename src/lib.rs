#[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
compile_error!(r#"sidefuzz currently only supports x86 and x86_64"#);

pub mod dudect;
pub mod optimizer;

use dudect::{DudeCT, DudeResult};
use optimizer::Optimizer;
use std::convert::TryFrom;
use std::mem::forget;
use std::ptr;
use wasmi::{ImportsBuilder, Module, ModuleInstance, NopExternals, RuntimeValue};

#[derive(Debug, Clone, Default)]
pub struct InputPair {
  pub first: Vec<u8>,
  pub second: Vec<u8>,
}

#[derive(Debug, Clone, Default)]
pub struct ScoredInputPair {
  pub score: f64,
  pub pair: InputPair,
}

pub struct SideFuzz {
  len: usize,
  wasm_module: Module,
}

impl SideFuzz {
  pub fn new(len: usize, wasm_module: Module) -> Self {
    SideFuzz { len, wasm_module }
  }

  pub fn run(&self) {

    // Instantiate a module with empty imports and
    // assert that there is no `start` function.
    let module_instance = ModuleInstance::new(&self.wasm_module, &ImportsBuilder::default())
      .expect("failed to instantiate wasm module")
      .assert_no_start();

    // Get memory instance exported by name 'mem' from the module instance.
    let internal_mem = module_instance.export_by_name("memory");
    let internal_mem = internal_mem.expect("Module expected to have 'mem' export");
    let internal_mem = internal_mem.as_memory().unwrap();

    let mut optimizer = Optimizer::new(self.len, |first: &[u8], second: &[u8]| {

      // First
      wasmi::reset_instruction_count();
      internal_mem.set(0, first).unwrap();
      module_instance
        .invoke_export(
          "sidefuzz",
          &[
            RuntimeValue::I32(0),
            RuntimeValue::I32(i32::try_from(first.len()).unwrap()),
          ],
          &mut NopExternals,
        )
        .expect("failed to execute export");

      let first_instructions = wasmi::get_instruction_count();

      // Second
      wasmi::reset_instruction_count();
      internal_mem.set(0, second).unwrap();
      module_instance
        .invoke_export(
          "sidefuzz",
          &[
            RuntimeValue::I32(0),
            RuntimeValue::I32(i32::try_from(second.len()).unwrap()),
          ],
          &mut NopExternals,
        )
        .expect("failed to execute export");

      let second_instructions = wasmi::get_instruction_count();

      // Difference
      let diff;
      if first_instructions >= second_instructions {
        diff = first_instructions - second_instructions;
      } else {
        diff = second_instructions - first_instructions;
      }

      diff as f64
    });

    println!("Evolving candidate input pairs");
    let mut best = ScoredInputPair::default(); // defaults to score of zero.
    loop {
      // Check results once every 1000 genearations
      for _ in 0..1000 {
        optimizer.step();
      }
      let population = optimizer.scored_population();
      let new_best = population[0].clone();

      println!(
        "{} {} {}",
        new_best.score,
        hex::encode(&population[0].pair.first),
        hex::encode(&population[0].pair.second)
      );

      if new_best.score > best.score {
        best = new_best;
      } else if best.score != 0.0 {
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

// FIXME: We don't have black_box in stable rust

/// A function that is opaque to the optimizer, to allow benchmarks to
/// pretend to use outputs to assist in avoiding dead-code
/// elimination.
///
/// NOTE: We don't have a proper black box in stable Rust. This is
/// a workaround implementation, that may have a too big performance overhead,
/// depending on operation, or it may fail to properly avoid having code
/// optimized out. It is good enough that it is used.
#[inline(never)]
pub fn black_box<D>(dummy: D) -> D {
  unsafe {
    let ret = ptr::read_volatile(&dummy);
    forget(dummy);
    ret
  }
}

fn p_value_from_t_value(t: f64) -> f64 {
  // TODO: use formula instead of table.

  if t <= 0.0 {
    return 1.0; // 0% confidence.
  }

  // Assume infinite degrees of freedom
  // Two tailed t test
  let t_table = vec![
    (10.000, 0.0), // 100% confidence
    (3.91, 0.0001),
    (3.291, 0.001),
    (3.090, 0.002),
    (2.807, 0.005),
    (2.576, 0.01),
    (2.326, 0.02),
    (1.960, 0.05),
    (1.645, 0.1),
    (1.282, 0.2),
    (1.036, 0.3),
    (0.842, 0.4),
    (0.674, 0.5),
    (0.253, 0.6),
    (0.0, 1.0), // 0% confidence
  ];

  for (t_value, p_value) in t_table {
    if t > t_value {
      return p_value;
    }
  }

  panic!("Invalid t value");
}
