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


// This section contains utility functions used by WASM targets
// ------------------------------------------------------------


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

#[cfg(any(target_arch = "wasm32"))]
static INPUT: &'static mut Vec<u8> = vec![];

#[cfg(any(target_arch = "wasm32"))]
static INPUT_SET: &'static mut bool = false;

/// Get an input of the desired length.
/// This function should be called with a constant unchanging len argument.
/// Calling it with different lengths will result in invalid fuzzing.
///
/// Example:
/// ```rust
/// let input = sidefuzz::input(32); // get 32 bytes of input
/// sidefuzz::black_box(my_contant_time_fn(input));
/// ```
///
// This is a VERY ODD fuction that provides us with a really nice external API.
// 1. It is called once before fuzzing starts in order to set the size of INPUT.
// 2. After it is called once, we call input_pointer and input_len from the host to get a stable pointer to INPUT.
// 3. Fuzzing starts, we write data to INPUT from the host, then call the exported `sidefuzz` function.
#[cfg(any(target_arch = "wasm32"))]
pub fn input(len: usize) -> &'static [u8] {
  if INPUT_SET == false {
    INPUT.resize(len, 0);
    INPUT_SET = true;
    panic!("Input length successfully set. Panicking to unwind and stop execution via wasm trap.");
  }

  INPUT
}

/// Get a pointer to the input array
/// This needs to be public so we can call it across host/wasm boundary,
/// but it should be considered a "private" function to sidefuzz.
/// It's API is not stable and may be subject to change
#[cfg(any(target_arch = "wasm32"))]
#[doc(hidden)]
#[no_mangle]
pub extern "C" fn input_pointer() -> i32 {
  unsafe { INPUT.as_ptr() as i32 }
}

/// Get a length of the input array
/// This needs to be public so we can call it across host/wasm boundary,
/// but it should be considered a "private" function to sidefuzz.
/// It's API is not stable and may be subject to change
#[cfg(any(target_arch = "wasm32"))]
#[doc(hidden)]
#[no_mangle]
pub extern "C" fn input_len() -> i32 {
  INPUT.len() as i32;
}