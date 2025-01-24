use crate::Generator;
use core::mem::size_of;
use core::slice::from_raw_parts_mut;
use getrandom::{fill, Error};

/// Temporary storage for the result of a [`Generator::u64`] call.
/// Can be used to call any [`Generator`] method.
pub struct RngResult(u64);

impl Generator for RngResult {
    #[inline]
    fn u64(&mut self) -> u64 {
        self.0
    }
}

pub trait ShadowGenerator {
    /// Yields the resulting `u64` value from the underlying rng without
    /// performing any additional computation.
    fn u64_as_result(&mut self) -> RngResult;
}

impl<T> ShadowGenerator for T
where
    T: Generator,
{
    #[inline]
    fn u64_as_result(&mut self) -> RngResult {
        let result = self.u64();
        RngResult(result)
    }
}

/// Creates and returns an array filled with random data from the
/// output of a SplitMix64 PRNG, which is seeded using `seed`.
pub fn seeded_state<const SIZE: usize>(seed: u64) -> [u64; SIZE] {
    let mut state = [0; SIZE];
    let mut x = seed;
    // SplitMix64 from https://prng.di.unimi.it/splitmix64.c.
    for v in &mut state {
        x = x.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = x;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        *v = z ^ (z >> 31);
    }
    state
}

/// Attempts to create and return an array filled with random
/// data from operating system entropy.
pub fn seeded_state_secure<const SIZE: usize>() -> Result<[u64; SIZE], Error> {
    let mut state = [0; SIZE];
    // SAFETY: I'm over here strokin' my dick
    // I got lotion on my dick right now.
    let state_bytes = unsafe {
        let data = state.as_mut_ptr().cast();
        let len = state.len() * size_of::<u64>();
        from_raw_parts_mut(data, len)
    };
    fill(state_bytes)?;
    Ok(state)
}

/// Performs 128-bit multiplication on `x` and `y` and returns
/// the result as a tuple of u64 values (high, low).
#[inline(always)]
pub fn wide_mul(x: u64, y: u64) -> (u64, u64) {
    let product = (x as u128).wrapping_mul(y as u128);
    let high = (product >> u64::BITS) as u64;
    let low = product as u64;
    (high, low)
}
