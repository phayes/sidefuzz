use crate::black_box;
use crate::cpucycles;
use rolling_stats::Stats;

#[derive(Eq, PartialEq, Debug)]
pub enum DudeResult {
    Ok,       // Success
    Err,      // Failure
    Progress, // Neither success nor failure, still in progress.
}

pub struct DudeCT<'a, T>
where
    T: Fn(&[u8]),
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
    T: Fn(&[u8]),
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
            t_threshold,
            t_fail,
            fail_min_samples,
            first,
            second,
            function,
            first_stats: Stats::new(),
            second_stats: Stats::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.first_stats.count + self.second_stats.count
    }

    pub fn sample(&mut self, num_samples: u64) -> (f64, DudeResult) {
        for _ in 0..num_samples {
            // randomly select which side to execute
            if rand::random() {
                let cycles_marker = cpucycles::cpucycles();
                black_box((self.function)(&self.first));
                let num_cycles = cpucycles::cpucycles() - cycles_marker;
                self.first_stats.update(num_cycles as f64);
            } else {
                let cycles_marker = cpucycles::cpucycles();
                black_box((self.function)(&self.second));
                let num_cycles = cpucycles::cpucycles() - cycles_marker;
                self.second_stats.update(num_cycles as f64);
            }
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
        let mut dudect = DudeCT::new(
            3.2905,    // Success t-value
            0.674,     // Give up t-value
            1_000_000, // Give up min samples
            &one,
            &ff,
            |input: &[u8]| {
                black_box(fibonacci(input[0]));
            },
        );

        let (_t, result) = dudect.sample(100_000);
        assert_eq!(result, DudeResult::Ok);
    }
}
