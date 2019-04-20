#[cfg(target_arch = "x86")]
use core::arch::x86::_rdtsc;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::_rdtsc;


pub fn cpucycles() -> u64 {
    let counter: i64 = unsafe { _rdtsc() };
    counter as u64
}
