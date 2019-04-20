#[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
compile_error!(r#"sidefuzz currently only supports x86 and x86_64"#);

pub mod cpucycles;
pub mod dudect;
pub mod optimizer;

use dudect::{DudeCT, DudeResult};
use optimizer::Optimizer;
use std::mem::forget;
use std::ptr;

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

pub struct SideFuzz<T>
where
  T: Fn(&[u8]) -> Result<(), ()>,
{
  len: usize,
  function: T,
}

impl<T> SideFuzz<T>
where
  T: Fn(&[u8]) -> Result<(), ()>,
{
  pub fn new(len: usize, function: T) -> Self {
    SideFuzz {
      len,
      function,
    }
  }

  fn num_executions(&self) -> u64 {
    // Find a good input
    let mut input: Vec<u8>;
    loop {
      input = (0..self.len).map(|_| rand::random::<u8>()).collect();
      let result = (self.function)(&input);
      if result.is_ok() {
        break;
      }
    }

    let mut num_executions = 1;
    loop {
      let timer = cpu_time::ProcessTime::now();
      for _ in 0..num_executions {
        let _ = black_box((self.function)(&input));
      }
      // Target of 10 at least microseconds per run
      let time = timer.elapsed().as_micros();
      if time > 10 {
        break;
      } else {
        num_executions = num_executions * 2;
      }
    }

    num_executions
  }

  pub fn run(&self) {
    // Pin to a single core
    // TODO: for multi-core checking, pin to different cores
    println!("Setting core affinity");
    let core_ids = core_affinity::get_core_ids().unwrap();
    core_affinity::set_for_current(core_ids[0]);

    // Sample execution time so that we have at least 10 microseconds per run
    println!("Determining number of executions per measurement");
    let num_executions = self.num_executions();
    println!("{} executions per measurement", num_executions);

    let mut time_optimizer = Optimizer::new(self.len, |first: &[u8], second: &[u8]| {
      let mut result: Result<(), ()> = Ok(());

      let cycles_marker = cpucycles::cpucycles();
      for _ in 0..num_executions {
        result = black_box((self.function)(first));
      }
      let first_cycles = cpucycles::cpucycles() - cycles_marker;
      if result.is_err() {
        return 0.0;
      }

      let cycles_marker = cpucycles::cpucycles();
      for _ in 0..num_executions {
        result = black_box((self.function)(second));
      }
      let second_cycles = cpucycles::cpucycles() - cycles_marker;
      if result.is_err() {
        return 0.0;
      }

      let ratio: f64 = first_cycles as f64 / second_cycles as f64;

      if ratio.is_nan() {
        return 0.0;
      }

      ratio
    });

    println!("Evolving candidate input pairs");
    let mut best = ScoredInputPair::default(); // defaults to score of zero.
    let mut average: f64 = 0.0;
    loop {
      // Check results once every 1000 genearations
      for _ in 0..1000 {
        time_optimizer.step();
      }
      let population = time_optimizer.scored_population();

      // Calculate average
      let sum: f64 = population.iter().fold(0.0, |mut sum, val| {
        sum += val.score;
        sum
      });
      let new_average = sum / (population.len() as f64);

      println!(
        "{} {} {}",
        new_average,
        hex::encode(&population[0].pair.first),
        hex::encode(&population[0].pair.second)
      );

      if new_average >= average {
        best = population[0].clone();
        average = new_average;
      } else {
        println!(
          "Checking {} {}",
          hex::encode(&best.pair.first),
          hex::encode(&best.pair.second)
        );

        // Construct DudeCT
        // Return success on t = 3.2905 (99.999 confidence)
        // Give up on t < 0.674 (50% confidence) when over 1 million samples.
        let mut dudect = DudeCT::new(
          3.2905,    // Success t-value
          0.674,     // Give up t-value
          1_000_000, // Give up min samples
          &best.pair.first,
          &best.pair.second,
          |input: &[u8]| {
            for _ in 0..num_executions {
              let _ = black_box(&(self.function)(input));
            }
            Ok(())
          },
        );

        loop {
          let (t, result) = dudect.sample(100_000);
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
                "Found timing difference between these two inputs with {}% confidence:\ninput 1: {}\ninput 2: {}",
                (1.0 - p) * 100.0,
                hex::encode(&best.pair.first),
                hex::encode(&best.pair.second)
              );
              std::process::exit(0);
            }
            DudeResult::Err => {
              best = ScoredInputPair::default();
              average = 0.0;
              println!("Giving up on these inputs. Continuing to evolve candidate inputs.");
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
    (0.0, 1.0), // 0% confidence
  ];

  for (t_value, p_value) in t_table {
    if t > t_value {
      return p_value;
    }
  }

  panic!("Invalid t value");
}
