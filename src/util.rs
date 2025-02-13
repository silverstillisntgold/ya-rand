use core::{mem::size_of, slice::from_raw_parts_mut};
use getrandom::{fill, Error};

/// Returns an array filled with pseudo-random data from the output of
/// a SplitMix64 PRNG, which is seeded using `seed`.
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
pub fn state_from_entropy<const SIZE: usize>() -> Result<[u64; SIZE], Error> {
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

#[cfg(all(feature = "std", feature = "secure"))]
#[inline]
pub fn text<E: crate::encoding::Encoding, const LEN: usize>(
    rng: &mut impl crate::SecureYARandGenerator,
) -> std::string::String {
    use std::{string::String, vec};
    const {
        assert!(LEN >= E::MIN_LEN);
    }
    let mut data = vec![0; LEN];
    rng.fill_bytes(&mut data);
    for i in 0..data.len() {
        data[i] = E::ALPHABET[(data[i] as usize) % E::ALPHABET.len()];
    }
    unsafe { String::from_utf8_unchecked(data) }
}

/// Performs 128-bit multiplication on `x` and `y` and returns
/// the result as a tuple of u64 values (high, low).
///
/// On modern architectures this can often be compiled
/// into a single instruction.
#[inline(always)]
pub fn wide_mul(x: u64, y: u64) -> (u64, u64) {
    let product = (x as u128).wrapping_mul(y as u128);
    let high = (product >> u64::BITS) as u64;
    let low = product as u64;
    (high, low)
}
