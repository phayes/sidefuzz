// An implementation of dudect
pub(crate) mod dudect;

// A genetic optimizer
pub(crate) mod optimizer;

// The fuzz command
#[cfg(not(any(target_arch = "wasm32")))]
#[doc(hidden)]
pub mod fuzz;

// The check command
#[cfg(not(any(target_arch = "wasm32")))]
#[doc(hidden)]
pub mod check;

// Wasm Module wrapper
pub(crate) mod wasm;

// Errors
pub(crate) mod errors;

// Misc utility functions and types
pub(crate) mod util;

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
    let ret = std::ptr::read_volatile(&dummy);
    std::mem::forget(dummy);
    ret
  }
}

