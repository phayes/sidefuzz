extern "C" {
    pub fn unsafe_cpucycles() -> libc::int64_t;
}

pub fn cpucycles() -> u64 {
    unsafe { unsafe_cpucycles() as u64 }
}
