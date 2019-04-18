mod cpucycles;
mod dudect;
mod optimizer;
use optimizer::Optimizer;

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

  pub fn count_cycles(&self, input: &[u8]) -> u64 {
    // Sample 5 times and take the average of the middle 3
    let mut i = 5;
    let mut cycle_counts: Vec<u64> = Vec::with_capacity(5);
    loop {
      let cycle_count = cpucycles::cpucycles();
      let result = (self.function)(input);
      cycle_counts.push(cpucycles::cpucycles() - cycle_count);
      if result.is_err() {
        continue;
      } else {
        i -= 1;
      }
      if i == 0 {
        break;
      }
    }

    cycle_counts.sort();

    // TODO: make this dynamic
    let average = (cycle_counts[1] + cycle_counts[2] + cycle_counts[3]) / 3;

    average
  }

  pub fn time_run(&self) {
    // Pin to a single core
    // TODO: for multi-core checking, pin to different cores
    let core_ids = core_affinity::get_core_ids().unwrap();
    core_affinity::set_for_current(core_ids[0]);

    let mut time_optimizer = Optimizer::new(self.len, |first: &[u8], second: &[u8]| {
      let timer = cpu_time::ProcessTime::now();
      let result = (self.function)(first);
      let first_time = timer.elapsed().as_nanos();
      if result.is_err() {
        return 0.0;
      }

      let timer = cpu_time::ProcessTime::now();
      let result = (self.function)(second);
      let second_time = timer.elapsed().as_nanos();
      if result.is_err() {
        return 0.0;
      }

      let ratio = first_time as f64 / second_time as f64;

      ratio
    });

    // Check results once every 1000 genearations
    let mut best = ScoredInputPair::default(); // defaults to score of zero.
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

      if population[0].score >= best.score {
        best = population[0].clone();
      } else {
        println!(
          "Checking {} {}",
          hex::encode(&best.pair.first),
          hex::encode(&best.pair.second)
        );
        let dudect = dudect::DudeCT::new(&best.pair.first, &best.pair.second, &self.function);
        dudect.time_run();

        break;
      }
    }
  }

  pub fn cycles_run(&self) {
    // Pin to a single core
    // TODO: for multi-core checking, pin to different cores
    let core_ids = core_affinity::get_core_ids().unwrap();
    core_affinity::set_for_current(core_ids[0]);

    let mut cycle_optimizer = Optimizer::new(self.len, |first: &[u8], second: &[u8]| {
      let first_cycles = self.count_cycles(first);
      let second_cycles = self.count_cycles(second);
      let ratio = first_cycles as f64 / second_cycles as f64;

      ratio
    });

    // Check results once every 1000 genearations
    let mut best = ScoredInputPair::default(); // defaults to score of zero.
    loop {
      for _ in 0..1000 {
        cycle_optimizer.step();
      }
      let population = cycle_optimizer.scored_population();
      println!(
        "{} {} {}",
        &population[0].score,
        hex::encode(&population[0].pair.first),
        hex::encode(&population[0].pair.second)
      );

      if population[0].score >= best.score {
        best = population[0].clone();
      } else {
        println!(
          "Checking {} {}",
          hex::encode(&best.pair.first),
          hex::encode(&best.pair.second)
        );
        let dudect = dudect::DudeCT::new(&best.pair.first, &best.pair.second, &self.function);
        dudect.time_run();

        break;
      }
    }
  }
}
