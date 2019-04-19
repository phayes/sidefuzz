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

  pub fn run(&self) -> Result<f64, ()> {
    let mut first_samples: Vec<f64> = Vec::new();
    let mut second_samples: Vec<f64> = Vec::new();

    let mut t: f64;
    loop {
      for _ in 0..100_000 {
        let timer = cpu_time::ProcessTime::now();
        (self.function)(&self.first).unwrap(); // Because inputs are verified, unwrap OK
        first_samples.push(timer.elapsed().as_nanos() as f64);

        let timer = cpu_time::ProcessTime::now();
        (self.function)(&self.second).unwrap(); // Because inputs are verified, unwrap OK
        second_samples.push(timer.elapsed().as_nanos() as f64);
      }

      t = calculate_t(&first_samples, &second_samples);

      let p = p_value_from_t_value(t);
      println!(
        "samples: {}, t-value: {}, confidence: {}%",
        first_samples.len(),
        t,
        (1.0 - p) * 100.0
      );

      // Return results when we get to 99.999% confidence.
      if t > 3.2905 {
        println!(
          "Found timing difference between these two inputs with {}% confidence:\ninput 1: {}\ninput 2: {}",
          (1.0 - p) * 100.0,
          hex::encode(self.first),
          hex::encode(self.second)
        );
        return Ok(t);
      }
      // If we have over a million samples, and still have less than 50% confidence, give up.
      if first_samples.len() > 1_000_000 && t < 0.674 {
        println!("Giving up on these inputs...");
        return Err(());
      }
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

  t.abs()
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
