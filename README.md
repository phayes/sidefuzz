# SideFuzz: Fuzzing for timing side-channel vulnerabilities using wasm

SideFuzz is an adaptive fuzzer that uses a genetic-algorithim optimizer in combination with t-statistics to find side-channel (timing) vulnerabilities in cryptography compiled to wasm.

Fuzzing Targets can be found here: https://github.com/phayes/sidefuzz-targets

### How it works

**Phase 1.** Uses a genetic-algorithim optimizer that tries to maximize the running time between two different inputs. It will continue optimizing until subsequent generations of input-pairs no longer produce any meaningful differences in running time. This means that it will optimize until it finds finds a "local maxima" in the fitness of input pairs.

**Phase 2.** Once a local optima is found, the leading input-pairs are sampled until either:

- A large t-statistic (p = 0.001) is found, indicating that there is a statistically significant difference in running-time between the two inputs. This is indicative of a timing side-channel vulnerability; or

- The t-statistic stays low, even after significant sampling. In this case the candidate input pairs are rejected and SideFuzz returns to phase 1, resuming the genetic-algorithim optimizer to find another local optmia.

The current version uses elapsed CPU cycles as it's measurement. Future versions will include PAPI support behind a feature flag.

### Furthur Reading

1. "DifFuzz: Differential Fuzzing for Side-Channel Analysis", Nilizadeh et al.
   https://arxiv.org/abs/1811.07005

2. "Dude, is my code constant time?", Reparaz et al. https://eprint.iacr.org/2016/1123.pdf

3. "Rust, dudect and constant-time crypto in debug mode", brycx.
   https://brycx.github.io/2019/04/21/rust-dudect-constant-time-crypto.html

### Related Tools

1. `dudect-bencher`. An implementation of the DudeCT constant-time function tester. In comparison to SideFuzz, this tool more closely adheres to the original dudect design. https://crates.io/crates/dudect-bencher

2. `ctgrind`. Tool for checking that functions are constant time using Valgrind. https://github.com/RustCrypto/utils/tree/master/ctgrind

## Rust

Creating a target in rust is very easy.

```rust
use std::ptr;
use std::slice;
use sidefuzz::black_box;

#[no_mangle]
pub extern "C" fn sidefuzz(ptr: i32, len: i32) {
  let input: &[u8] = unsafe { slice::from_raw_parts(ptr as _, len as _) };
  black_box(hopefully_constant_fn(input));
}
```

You would then compile and fuzz your target like so:

```bash
rustup target add wasm32-unknown-unknown                          # Only needs to be done once.
cargo build --release                                             # Always pass the release flag
sidefuzz 32 ./target/wasm32-unknown-unknown/release/mytarget.wasm # Fuzz with 32 bytes of input
```

## Other Languages

SideFuzz works with Go, C, C++ and other langauges that compile to wasm.

The wasm module should provide two exports: 

1. Memory exported to "memory"

2. A function named "sidefuzz" that takes two `i32` arguments. The first argument is a pointer to the start of the input array, the second argument is the length of the input array. 

The `sidefuzz` function will be called repeatedly during the fuzzing process. 
