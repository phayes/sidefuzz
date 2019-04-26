
use rolling_stats::Stats;
use std::convert::TryFrom;
use wasmi::{ImportsBuilder, Module, ModuleInstance, NopExternals, RuntimeValue};
#[derive(Eq, PartialEq, Debug)]
pub enum DudeResult {
    Ok,       // Success
    Err,      // Failure
    Progress, // Neither success nor failure, still in progress.
}

pub struct DudeCT<'a> {
    t_threshold: f64,
    t_fail: f64,
    fail_min_samples: usize,
    first: &'a [u8],
    second: &'a [u8],
    wasm_module: &'a Module,
    first_stats: Stats<f64>,
    second_stats: Stats<f64>,
}

impl<'a> DudeCT<'a> {
    pub fn new(
        t_threshold: f64,
        t_fail: f64,
        fail_min_samples: usize,
        first: &'a [u8],
        second: &'a [u8],
        wasm_module: &'a Module,
    ) -> Self {

        // Todo: Do a single instantiation of the module
        // 1. Check the lengths.
        // 2. Check to make sure it doesn't trap.

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
        // TODO: Because the module might secretly store state, we should not panic, and instead resturn a Result.

        // Instantiate a module with empty imports and
        // assert that there is no `start` function.
        let module_instance = ModuleInstance::new(&self.wasm_module, &ImportsBuilder::default())
            .expect("failed to instantiate wasm module")
            .assert_no_start();

        // Get memory instance exported by name 'mem' from the module instance.
        let internal_mem = module_instance.export_by_name("memory");
        let internal_mem = internal_mem.expect("Module expected to have 'memory' export");
        let internal_mem = internal_mem.as_memory().unwrap();

        for _ in 0..num_samples {
            // First
            wasmi::reset_instruction_count();
            internal_mem.set(0, self.first).unwrap();
            let result = module_instance.invoke_export(
                "sidefuzz",
                &[
                    RuntimeValue::I32(0),
                    RuntimeValue::I32(i32::try_from(self.first.len()).unwrap()),
                ],
                &mut NopExternals,
            );
            result.expect("WASM module trapped. This is a bug, please report it to https://github.com/phayes/sidefuzz");

            let first_instructions = wasmi::get_instruction_count();
            self.first_stats.update(first_instructions as f64);

            // Second
            wasmi::reset_instruction_count();
            internal_mem.set(0, self.second).unwrap();
            let result = module_instance.invoke_export(
                "sidefuzz",
                &[
                    RuntimeValue::I32(0),
                    RuntimeValue::I32(i32::try_from(self.second.len()).unwrap()),
                ],
                &mut NopExternals,
            );
            result.expect("WASM module trapped. This is a bug, please report it to https://github.com/phayes/sidefuzz");

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
    // TODO: Create a test that uses a minimal hard-coded WAT.
}
