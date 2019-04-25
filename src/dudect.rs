use wasmi::{ImportsBuilder, Module, ModuleInstance, NopExternals, RuntimeValue};
use rolling_stats::Stats;
use std::convert::TryFrom;

#[derive(Eq, PartialEq, Debug)]
pub enum DudeResult {
    Ok,       // Success
    Err,      // Failure
    Progress, // Neither success nor failure, still in progress.
}

pub struct DudeCT<'a>
{
    t_threshold: f64,
    t_fail: f64,
    fail_min_samples: usize,
    first: &'a [u8],
    second: &'a [u8],
    wasm_module: &'a Module,
    first_stats: Stats<f64>,
    second_stats: Stats<f64>,
}

impl<'a> DudeCT<'a>
{
    pub fn new(
        t_threshold: f64,
        t_fail: f64,
        fail_min_samples: usize,
        first: &'a [u8],
        second: &'a [u8],
        wasm_module: &'a Module,
    ) -> Self {
        DudeCT {
            t_threshold,
            t_fail,
            fail_min_samples,
            first,
            second,
            wasm_module,
            first_stats: Stats::new(),
            second_stats: Stats::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.first_stats.count + self.second_stats.count
    }

    pub fn sample(&mut self, num_samples: u64) -> (f64, DudeResult) {
        // Instantiate a module with empty imports and
        // assert that there is no `start` function.
        let module_instance = ModuleInstance::new(&self.wasm_module, &ImportsBuilder::default())
            .expect("failed to instantiate wasm module")
            .assert_no_start();

    // Get memory instance exported by name 'mem' from the module instance.
    let internal_mem = module_instance.export_by_name("memory");
    let internal_mem = internal_mem.expect("Module expected to have 'mem' export");
    let internal_mem = internal_mem.as_memory().unwrap();

        for _ in 0..num_samples {
      // First
      wasmi::reset_instruction_count();
      internal_mem.set(0, self.first).unwrap();
      module_instance
        .invoke_export(
          "sidefuzz",
          &[
            RuntimeValue::I32(0),
            RuntimeValue::I32(i32::try_from(self.first.len()).unwrap()),
          ],
          &mut NopExternals,
        )
        .expect("failed to execute export");

      let first_instructions = wasmi::get_instruction_count();
      self.first_stats.update(first_instructions as f64);

      // Second
      wasmi::reset_instruction_count();
      internal_mem.set(0, self.second).unwrap();
      module_instance
        .invoke_export(
          "sidefuzz",
          &[
            RuntimeValue::I32(0),
            RuntimeValue::I32(i32::try_from(self.second.len()).unwrap()),
          ],
          &mut NopExternals,
        )
        .expect("failed to execute export");

      let second_instructions = wasmi::get_instruction_count();
      self.second_stats.update(second_instructions as f64);

        }

        let t = calculate_t(&self.first_stats, &self.second_stats);

        // Return results when t value is above threshold
        if t >= self.t_threshold {
            (t, DudeResult::Ok)
        }
        // Check if we should give up
        else if self.first_stats.count > self.fail_min_samples && t <= self.t_fail {
            (t, DudeResult::Err)
        } else {
            // Neither success nor failure, keep going.
            (t, DudeResult::Progress)
        }
    }
}

fn calculate_t(first: &Stats<f64>, second: &Stats<f64>) -> f64 {
    let first_mean = first.mean;
    let second_mean = second.mean;

    let first_std_dev = first.std_dev;
    let second_std_dev = first.std_dev;

    let first_variance = first_std_dev * first_std_dev;
    let second_variance = second_std_dev * second_std_dev;

    let first_sample_size = first.count as f64;
    let second_sample_size = second.count as f64;

    let t = (first_mean - second_mean)
        / ((first_variance / first_sample_size) + (second_variance / second_sample_size)).sqrt();

    t.abs()
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn dudect_test() {
        pub fn fibonacci(n: u8) -> f64 {
            if n == 0 {
                panic!("zero is not a right argument to fibonacci()!");
            } else if n == 1 {
                return 1.0;
            }

            let mut sum = 0.0;
            let mut last = 0.0;
            let mut curr = 1.0;
            for _i in 1..n {
                sum = last + curr;
                last = curr;
                curr = sum;
            }
            sum
        }

        let one = vec![1u8];
        let ff = vec![255u8];
        //let mut dudect = DudeCT::new(
        //    3.2905,    // Success t-value
        //    0.674,     // Give up t-value
        //    1_000_000, // Give up min samples
        //    &one,
        //    &ff,
        //    |input: &[u8]| {
        //        black_box(fibonacci(input[0]));
        //    },
        //);

        //let (_t, result) = dudect.sample(100_000);
        //assert_eq!(result, DudeResult::Ok);
    }
}
