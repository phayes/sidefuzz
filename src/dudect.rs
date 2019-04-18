use crate::cpucycles;
use statrs::statistics::Mean;
use statrs::statistics::Variance;

pub struct DudeCT<'a, T>
where
  T: Fn(&[u8]) -> Result<(), ()>,
{
  first: &'a [u8],
  second: &'a [u8],
  function: T,
}

impl<'a, T> DudeCT<'a, T>
where
  T: Fn(&[u8]) -> Result<(), ()>,
{
  pub fn new(first: &'a [u8], second: &'a [u8], function: T) -> Self {
    DudeCT {
      first: first,
      second: second,
      function: function,
    }
  }

  pub fn time_run(&self) {
    let mut first_samples: Vec<f64> = Vec::new();
    let mut second_samples: Vec<f64> = Vec::new();

    loop {
      for _ in 0..100_000 {
        let timer = cpu_time::ProcessTime::now();
        (self.function)(&self.first).unwrap(); // Because inputs are verified, unwrap OK
        first_samples.push(timer.elapsed().as_nanos() as f64);

        let timer = cpu_time::ProcessTime::now();
        (self.function)(&self.second).unwrap(); // Because inputs are verified, unwrap OK
        second_samples.push(timer.elapsed().as_nanos() as f64);
      }

      let t = calculate_t(&first_samples, &second_samples);

      println!("samples: {}, t-value: {}", first_samples.len(), t);
    }
  }

  pub fn cycle_run(&self) {
    let mut first_samples: Vec<f64> = Vec::new();
    let mut second_samples: Vec<f64> = Vec::new();

    loop {
      for _ in 0..100_000 {
        let cycle_count = cpucycles::cpucycles();
        (self.function)(&self.first).unwrap(); // Because inputs are verified, unwrap OK
        first_samples.push((cpucycles::cpucycles() - cycle_count) as f64);

        let cycle_count = cpucycles::cpucycles();
        (self.function)(&self.second).unwrap(); // Because inputs are verified, unwrap OK
        second_samples.push((cpucycles::cpucycles() - cycle_count) as f64);
      }

      let t = calculate_t(&first_samples, &second_samples);

      println!("samples: {}, t-value: {}", first_samples.len(), t);
    }
  }
}

fn calculate_t(first: &[f64], second: &[f64]) -> f64 {
  debug_assert!(first.len() == second.len());

  let first_mean = first.mean();
  let second_mean = second.mean();

  let first_variance = first.variance();
  let second_variance = second.variance();

  let sample_size = first.len() as f64;

  let t = (first_mean - second_mean)
    / ((first_variance / sample_size) + (second_variance / sample_size)).sqrt();

  t
}
