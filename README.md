## SideFuzz: Fuzzing for timing side-channel vulnerabilities

SideFuzz is an adaptive fuzzer that uses a genetic-algorithim optimizer in combination with t-statistics to find side-channel (timing) vulnerabilities in cryptography compiled to [wasm](https://webassembly.org).

Fuzzing Targets can be found here: https://github.com/phayes/sidefuzz-targets

### How it works

SideFuzz works by counting instructions executed in the [wasmi](https://github.com/paritytech/wasmi) wasm interpreter. It works in two phases:

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
// lib.rs

#[no_mangle]
pub extern "C" fn fuzz() {
  let input = sidefuzz::get_input(32); // 32 bytes of of fuzzing input as a &[u8]
  sidefuzz::black_box(my_hopefully_constant_fn(input));
}
```

```toml
# Cargo.toml

[lib]
crate-type = ["cdylib"]

[dependencies]
sidefuzz = {git = "https://github.com/phayes/sidefuzz"}
```

Compile and fuzz the target like so:

```bash
cargo build --release --target wasm32-unknown-unknown                # Always build in release mode
sidefuzz fuzz ./target/wasm32-unknown-unknown/release/my_target.wasm # Fuzzing!
```

Results can be checked like so:

```bash
sidefuzz check my_target.wasm 01250bf9 ff81f7b3
```

When fixing variable-time code, sidefuzz can also help with `sidefuzz count` to quickly count the number of instructions executed by the target.

```bash
cargo build --release && sidefuzz count my_target.wasm 01250bf9
```

## Other Languages

SideFuzz works with Go, C, C++ and other langauges that compile to wasm.

The wasm module should provide four exports:

1. Memory exported to "memory"

2. A function named "fuzz". This function will be repeatedly called during the fuzzing process.

3. A function named "input_pointer" that returns an i32 pointer to a location in linear memory where we can can write an array of input bytes. The "fuzz" fuction should read this array of bytes as input for it's fuzzing.

4. A function named "input_len" that returns an i32 with the desired length of input in bytes.

## Installation

```
rustup target add wasm32-unknown-unknown
git clone git@github.com:phayes/sidefuzz.git
cd sidefuzz && cargo install --path .
```

## FAQ

#### 1. Why wasm?

Web Assembly allows us to precisely track the number of instructions executed, the type of instructions executed, and the amount of memory used. This is much more precise than other methods such as tracking wall-time or counting CPU cycles.

#### 2. Why do I alway need to build in release mode?

Many constant-time functions include calls to variable-time `debug_assert!()` functions that get removed during a release build. Rust's and LLVM optimizer may also mangle supposedly constant-time code in the name of optimization, introducing subtle timing vulnerabilities. Runnig in release mode let's us surface these issues.

#### 3. I need an RNG (Random Number Generator). What do?

You should make use of a PRNG with a static seed. While this is a bad idea for production code, it's great for fuzzing. See the [rsa_encrypt_pkcs1v15_message target](https://github.com/phayes/sidefuzz-targets/blob/master/rsa_encrypt_pkcs1v15_message/src/lib.rs) for an example on how to do this.

#### 4. What's up with `black_box` ?

`sidefuzz::black_box` is used to avoid dead-code elimination. Becauee we are interested in exercising the fuzzed code instead of getting results from it, the exported `sidefuzz` function doesn't return anything. The Rust optimizer sees all functions that don't return as dead-code and will try to eliminate them as part of it's optimizations. `black_box` is a function that is opaque to the optimizer, allowing us to exercise functions that don't return without them being optimized away. It should be used whenever calling a function that doesn't return anything or where we are ignoring the output returned.

#### 5. The fuzzer gave me invalid inputs, what now?

You should panic (causing a wasm trap). This will signal to the fuzzer that the inputs are invalid.

#### 6. I need to do some variable-time set-up. How do I do that?

You should use [`lazy_static`](https://crates.io/crates/lazy_static) to do any set-up work (like generating keys etc). The target is always run once to prime lazy statics before the real fuzzing starts.
