# SideFuzz: Fuzzing for timing side-channel vulnerabilities

SideFuzz is an adaptive fuzzer that uses a genetic-algorithim optimizer in combination with t-statistics to find side-channel (timing) vulnerabilities in cryptography.

Fuzzing Targets can be found here: https://github.com/phayes/sidefuzz-targets

**SideFuzz is a work in progress. There could be bugs that may result in inaccurate results. For example, see https://github.com/phayes/sidefuzz/issues/8**

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


## Usage

Using the fuzzer is incredibly easy. 

```rust
fn main() {
    let input_len = 32; // 32 byte (256 bit) input
    let fuzzer = sidefuzz::SideFuzz::new(input_len, #[inline(never)]
    |input: &[u8]| {
        sidefuzz::black_box(hopefully_constant_fn(input));
    });

    fuzzer.run();
}
```

## Known Issues

The fuzzer tends to produce false-positives with `opt-level = 3 `. I have yet to determine if this is due to a problem in the fuzzer, or if rust is acutally unable to produce good constant-time code when `opt-level = 3 ` is set.  In order to avoid this, add the following to your Cargo.toml in your target project:

```toml
[profile.release]
opt-level = 2
```

You should then compile with `cargo build --release`.  Do not use debug (`cargo build`) for fuzzing since your target may include `assert_debug!` calls that are not constant time.
