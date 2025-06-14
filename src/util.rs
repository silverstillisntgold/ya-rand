use core::mem::size_of;
use core::slice::from_raw_parts_mut;
use getrandom::{Error, fill};

/// Utility function for converting `slice` into a mutable slice of bytes.
///
/// # Safety
///
/// `T` must be valid as nothing more than a collection of bytes.
#[inline]
pub unsafe fn as_raw_bytes_mut<T>(slice: &mut [T]) -> &mut [u8] {
    unsafe {
        let data = slice.as_mut_ptr().cast();
        let len = slice.len() * size_of::<T>();
        from_raw_parts_mut(data, len)
    }
}

/// Returns an array filled with pseudo-random data from the output of
/// a SplitMix64 PRNG, which is seeded using `seed`.
#[inline]
pub fn state_from_seed<const SIZE: usize>(seed: u64) -> [u64; SIZE] {
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

/// Attempts to return an array filled with random data from operating system entropy.
#[inline]
pub fn state_from_entropy<const SIZE: usize>() -> Result<[u64; SIZE], Error> {
    let mut state = [0; SIZE];
    // SAFETY: I'm over here strokin' my dick I got lotion on my dick right now.
    let state_as_bytes = unsafe { as_raw_bytes_mut(&mut state) };
    fill(state_as_bytes)?;
    Ok(state)
}

/// Performs 128-bit multiplication on `x` and `y` and returns the
/// result as a tuple of `u64` values in the format (high, low).
#[inline]
pub fn wide_mul(x: u64, y: u64) -> (u64, u64) {
    let product = (x as u128).wrapping_mul(y as u128);
    let high = (product >> u64::BITS) as u64;
    let low = product as u64;
    (high, low)
}
