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

      println!("samples: {}, t-value: {}", first_samples.len(), t);

      if t > 3.2905 {
        println!(
          "Found timing difference with 99.99% confidence.\n{}\n{}",
          hex::encode(self.first),
          hex::encode(self.second)
        );
        return Ok(t);
      }
      if first_samples.len() > 1_000_000 && t < 0.253347 {
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

  t
}

fn degrees_freedom(first: &[f64], second: &[f64]) -> f64 {
  debug_assert!(first.len() == second.len());
  debug_assert!(first.len() >= 2);

  (first.len() - 1) as f64 + (second.len() - 1) as f64
}

fn p_value_from_t_value(t: f64) -> f64 {
  // TODO: use formula instead of table.

  // assume infinite degrees of freedom
  let t_table = vec![
    (3.2905, 0.0005),
    (2.57583, 0.005),
    (2.32635, 0.01),
    (1.95996, 0.025),
    (1.644854, 0.05),
    (1.281552, 0.10),
    (0.674490, 0.25),
    (0.253347, 0.4),
    (0.0, 0.0),
  ];

  for (t_value, p_value) in t_table {
    if t > t_value {
      return p_value;
    }
  }

  panic!("Invalid t value");
}
