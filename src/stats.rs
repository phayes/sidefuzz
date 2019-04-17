use statrs::statistics::Mean;
use statrs::statistics::Variance;

pub fn calculate_t(first: &[f64], second: &[f64]) -> f64 {
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

// Increase sample size and evalulate two different null hypothesis:
// 1. populations are the same
