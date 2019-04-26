
#[cfg(not(any(target_arch = "wasm32")))]
pub mod dudect;

#[cfg(not(any(target_arch = "wasm32")))]
pub mod optimizer;

#[cfg(not(any(target_arch = "wasm32")))]
pub mod sidefuzz;

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
  pub highest: f64,
  pub lowest: f64,
  pub pair: InputPair,
}

/// Given a t-value, the the p-value from it.
///
/// This currently uses t-tables, in the future it will use an actual formula.
pub fn p_value_from_t_value(t: f64) -> f64 {
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

