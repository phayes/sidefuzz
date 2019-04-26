## SideFuzz: Fuzzing for timing side-channel vulnerabilities

SideFuzz is an adaptive fuzzer that uses a genetic-algorithim optimizer in combination with t-statistics to find side-channel (timing) vulnerabilities in cryptography compiled to [wasm](https://webassembly.org).

Fuzzing Targets can be found here: https://github.com/phayes/sidefuzz-targets

### How it works

SideFuzz works by counting wasm instructions executed in the [wasmi](https://github.com/paritytech/wasmi) wasm interpreter.

**Phase 1.** Uses a genetic-algorithim optimizer that tries to maximize the difference in instructions executed between two different inputs. It will continue optimizing until subsequent generations of input-pairs no longer produce any meaningful differences in the number of instructions executed. This means that it will optimize until it finds finds a local optimum in the fitness of input pairs.

**Phase 2.** Once a local optimum is found, the leading input-pairs are sampled until either:

- A large t-statistic (p = 0.001) is found, indicating that there is a statistically significant difference in running-time between the two inputs. This is indicative of a timing side-channel vulnerability; or

- The t-statistic stays low, even after significant sampling. In this case the candidate input pairs are rejected and SideFuzz returns to phase 1, resuming the genetic-algorithim optimizer to find another local optimum.

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

Compile and fuzz the target like so:

```bash
rustup target add wasm32-unknown-unknown                          # Only needs to be done once.
cargo build cargo build --release --target wasm32-unknown-unknown # Always build in release mode
sidefuzz 32 ./target/wasm32-unknown-unknown/release/mytarget.wasm # Fuzz with 32 bytes of input
```

## Other Languages

SideFuzz works with Go, C, C++ and other langauges that compile to wasm.

The wasm module should provide two exports:

1. Memory exported to "memory"

2. A function named "sidefuzz" that takes two `i32` arguments. The first argument is a pointer to the start of the input array, the second argument is the length of the input array.

The `sidefuzz` function will be called repeatedly during the fuzzing process.

## FAQ

#### 1. Why wasm?

Web Assembly allows us to precisely track the number of instructions executed, the type of instructions executed, and the amount of memory used. This is much more precise than other methods such as tracking wall-time or counting CPU cycles.

#### 2. Why do I alway need to build in release mode?

Many constant-time functions include calls to variable-time `debug_assert!()` functions that get removed during a release build. Rust's and LLVM optimizer may also mangle supposedly constant-time code in the name of optimization, introducing subtle timing vulnerabilities. Runnig in release mode let's us surface these issues.

#### 3. I need an RNG (Random Number Generator). What do?

You should make use of a PRNG with a static seed. While this is a bad idea for production code, it's great for fuzzing.

#### 4. What's up with `black_box` ?

`sidefuzz::black_box` is used to avoid dead-code elimination. Becauee we are interested in exercising the fuzzed code instead of getting results from it, the exported `sidefuzz` function doesn't return anything. The Rust optimizer sees all functions that don't return as dead-code and will try to eliminate them as part of it's optimizations. `black_box` is a function that is opaque to the optimizer, allowing us to exercise functions that don't return without them being optimized away. It should be used whenever calling a function that doesn't return anything or where we are ignoring the output returned.
