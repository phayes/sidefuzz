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
    first_samples: Vec<f64>,
    second_samples: Vec<f64>,
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
            first_samples: Vec::new(),
            second_samples: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.first_samples.len()
    }

    pub fn sample(&mut self, num_samples: u64) -> (f64, DudeResult) {
        for _ in 0..num_samples {
            let timer = cpu_time::ProcessTime::now();
            (self.function)(&self.first).unwrap();
            self.first_samples.push(timer.elapsed().as_nanos() as f64);

            let timer = cpu_time::ProcessTime::now();
            (self.function)(&self.second).unwrap();
            self.second_samples.push(timer.elapsed().as_nanos() as f64);
        }

        let t = calculate_t(&self.first_samples, &self.second_samples);

        // Return results when t value is above threshold
        if t >= self.t_threshold {
            return (t, DudeResult::Ok);
        }
        // Check if we should give up
        else if self.first_samples.len() > self.fail_min_samples && t <= self.t_fail {
            return (t, DudeResult::Err);
        } else {
            // Neither success nor failure, keep going.
            return (t, DudeResult::Progress);
        }
    }
}

fn calculate_t(first: &[f64], second: &[f64]) -> f64 {
    debug_assert!(first.len() == second.len());

    let first_mean = first.mean();
    let second_mean = second.mean();

    let first_variance = first.variance();
    let second_variance = second.variance();

    let sample_size = first.len() as f64;

    let t = (first_mean - second_mean)
        / ((first_variance / sample_size) + (second_variance / sample_size)).sqrt();

    t.abs()
}
