mod cpucycles;
mod optimizer;
mod stats;
use optimizer::Optimizer;

#[derive(Debug, Clone)]
pub struct InputPair {
  pub first: Vec<u8>,
  pub second: Vec<u8>
}

#[derive(Debug, Clone)]
pub struct ScoredInputPair {
  pub score: f64,
  pub pair: InputPair
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

  pub fn run(&self) {
    let mut time_optimizer = Optimizer::new(self.len, |first: &[u8], second: &[u8]| {
      let cycle_count = cpucycles::cpucycles();
      let result = (self.function)(first);
      let first_cycles = cpucycles::cpucycles() - cycle_count;
      if result.is_err() {
        return 0.0;
      }

      let cycle_count = cpucycles::cpucycles();
      let result = (self.function)(second);
      let second_cycles = cpucycles::cpucycles() - cycle_count;
      if result.is_err() {
        return 0.0;
      }

      let ratio = first_cycles as f64 / second_cycles as f64;

      ratio
    });

    // Print results once every 1000 genearations
    loop {
      for _ in 0..1000 {
        time_optimizer.step();
      }
      let population = time_optimizer.scored_population();
      println!(
        "{} {} {}",
        &population[0].score,
        hex::encode(&population[0].pair.first),
        hex::encode(&population[0].pair.second)
      );
    }
  }
}
