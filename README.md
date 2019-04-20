# SideFuzz: Fuzzing for timing side-channel vulnerabilities

SideFuzz is an adaptive fuzzer that uses an genetic-algorithim optimizer in combination with t-statistics to find side-channel (timing) vulnerabilities in cryptography.

### How it works

**Phase 1.** Uses a genetic-algorithim optimizer that tries to maximize the running time between two different inputs. It will continue optimizing until subsequent generations of input-pairs no longer produce any meaningful differences in running time. This means that it will optmize until it finds finds a "local maxima" in the fitness of input pairs.

**Phase 2.** Once a local optima is found, the leading input-pairs are sampled until either:

- A large t-statistic (p = 0.001) is found, indicating that there is a statistically significant difference in running-time between the two inputs. This is indicative of a timing side-channel vulnerability; or

- The t-statistic stays low, even after significant sampling. In this case the candidate input pairs are rejected and SideFuzz returns to phase 1, resuming the genetic-algorithim optimizer to find another local optmia.

The current version uses elapsed CPU cycles as it's measurement. Future versions will include PAPI support behind a feature flag.

### Furthur Reading

1. "DifFuzz: Differential Fuzzing for Side-Channel Analysis", Nilizadeh et al.
   https://arxiv.org/abs/1811.07005

2. "Dude, is my code constant time?", Reparaz et al. https://eprint.iacr.org/2016/1123.pdf
