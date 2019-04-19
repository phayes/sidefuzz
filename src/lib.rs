mod cpucycles;
mod dudect;
mod optimizer;
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
      len: len,
      function: function,
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

      let ratio = first_cycles as f64 / second_cycles as f64;

      if ratio == std::f64::NAN {
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
        let dudect = dudect::DudeCT::new(&best.pair.first, &best.pair.second, |input: &[u8]| {
          for _ in 0..num_executions {
            let _ = black_box(&(self.function)(input));
          }
          Ok(())
        });
        let result = dudect.run();
        if result.is_ok() {
          break;
        } else {
          best = ScoredInputPair::default();
          average = 0.0;
          println!("Continuing to evolve candidate input pairs");
        }
      }
    }
  }
}

// FIXME: We don't have black_box in stable rust

/// NOTE: We don't have a proper black box in stable Rust. This is
/// a workaround implementation, that may have a too big performance overhead,
/// depending on operation, or it may fail to properly avoid having code
/// optimized out. It is good enough that it is used by default.
///
/// A function that is opaque to the optimizer, to allow benchmarks to
/// pretend to use outputs to assist in avoiding dead-code
/// elimination.
pub fn black_box<T>(dummy: T) -> T {
  unsafe {
    let ret = ptr::read_volatile(&dummy);
    forget(dummy);
    ret
  }
}
