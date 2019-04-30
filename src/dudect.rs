// Contains an implementation of dudect

use crate::errors::SideFuzzError;
use crate::wasm::WasmModule;
use rolling_stats::Stats;

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
    module: WasmModule,
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
        module: WasmModule,
    ) -> Result<Self, SideFuzzError> {

        if module.fuzz_len() != first.len() || module.fuzz_len() != second.len() {
            return Err(SideFuzzError::InputsDifferentSizes)
        }

        Ok(DudeCT {
            t_threshold,
            t_fail,
            fail_min_samples,
            first,
            second,
            module,
            first_stats: Stats::new(),
            second_stats: Stats::new(),
        })
    }

    pub fn len(&self) -> usize {
        self.first_stats.count + self.second_stats.count
    }

    pub fn sample(&mut self, num_samples: u64) -> Result<(f64, DudeResult), SideFuzzError> {
        for _ in 0..num_samples {
            let first_instructions = self.module.count_instructions(self.first)?;
            let second_instructions = self.module.count_instructions(self.second)?;
            self.first_stats.update(first_instructions as f64);
            self.second_stats.update(second_instructions as f64);
        }

        let t = calculate_t(&self.first_stats, &self.second_stats);

        // Return results when t value is above threshold
        if t >= self.t_threshold {
            Ok((t, DudeResult::Ok))
        }
        // Check if we should give up
        else if self.first_stats.count > self.fail_min_samples && t <= self.t_fail {
            Ok((t, DudeResult::Err))
        } else {
            // Neither success nor failure, keep going.
            Ok((t, DudeResult::Progress))
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
