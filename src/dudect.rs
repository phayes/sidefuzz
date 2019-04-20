use rolling_stats::Stats;
use statrs::statistics::Mean;
use statrs::statistics::Variance;

pub enum DudeResult {
    Ok,       // Success
    Err,      // Failure
    Progress, // Neither success nor failure, still in progress.
}

pub struct DudeCT<'a, T>
where
    T: Fn(&[u8]) -> Result<(), ()>,
{
    t_threshold: f64,
    t_fail: f64,
    fail_min_samples: usize,
    first: &'a [u8],
    second: &'a [u8],
    function: T,
    first_stats: Stats<f64>,
    second_stats: Stats<f64>,
}

impl<'a, T> DudeCT<'a, T>
where
    T: Fn(&[u8]) -> Result<(), ()>,
{
    pub fn new(
        t_threshold: f64,
        t_fail: f64,
        fail_min_samples: usize,
        first: &'a [u8],
        second: &'a [u8],
        function: T,
    ) -> Self {
        DudeCT {
            t_threshold: t_threshold,
            t_fail: t_fail,
            fail_min_samples: fail_min_samples,
            first: first,
            second: second,
            function: function,
            first_stats: Stats::new(),
            second_stats: Stats::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.first_stats.count
    }

    pub fn sample(&mut self, num_samples: u64) -> (f64, DudeResult) {
        for _ in 0..num_samples {
            let timer = cpu_time::ProcessTime::now();
            (self.function)(&self.first).unwrap();
            self.first_stats.update(timer.elapsed().as_nanos() as f64);

            let timer = cpu_time::ProcessTime::now();
            (self.function)(&self.second).unwrap();
            self.second_stats.update(timer.elapsed().as_nanos() as f64);
        }

        let t = calculate_t(&self.first_stats, &self.first_stats);

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
    debug_assert!(first.count == second.count);

    let first_mean = first.mean;
    let second_mean = second.mean;

    let first_std_dev = first.std_dev;
    let second_std_dev = first.std_dev;

    let first_variance = first_std_dev * first_std_dev;
    let second_variance = second_std_dev * second_std_dev;

    let sample_size = first.count as f64;

    let t = (first_mean - second_mean)
        / ((first_variance / sample_size) + (second_variance / sample_size)).sqrt();

    t.abs()
}
