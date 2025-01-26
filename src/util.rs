use core::{mem::size_of, slice::from_raw_parts_mut};
use getrandom::{fill as get_random, Error};

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

/// The Windows 10 RNG is infallible, and the [`getrandom`] internals
/// reflect this since version 0.3.0. But the backend functions aren't
/// inlined so rustc is unable to eliminate the error branch in user
/// code; we do it manually here.
#[inline(always)]
pub fn fill(dest: &mut [u8]) -> Result<(), Error> {
    if cfg!(all(windows, not(target_vendor = "win7"))) {
        unsafe {
            get_random(dest).unwrap_unchecked();
        }
    } else {
        get_random(dest)?;
    }
    Ok(())
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
