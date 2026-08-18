/// Converts `slice` into a mutable slice of bytes, providing access
/// to the raw data of the underlying buffer.
///
/// # Safety
///
/// `T` must be valid as nothing more than a collection of bytes.
#[inline]
pub unsafe fn as_bytes_mut<T>(slice: &mut [T]) -> &mut [u8] {
    unsafe {
        let data = slice.as_mut_ptr().cast();
        let len = size_of_val(slice);
        core::slice::from_raw_parts_mut(data, len)
    }
}

/// Returns an array filled with pseudorandom data from the output
/// of a SplitMix64 PRNG, which is seeded using `seed`.
#[inline]
pub fn state_from_seed<const SIZE: usize>(seed: u64) -> [u64; SIZE] {
    const {
        assert!(SIZE != 0);
    }

    let mut state = [0; _];
    let mut x = seed;

    // SplitMix64 implementation from https://prng.di.unimi.it/splitmix64.c.
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
pub fn state_from_entropy<const SIZE: usize>() -> Result<[u64; SIZE], getrandom::Error> {
    const {
        assert!(SIZE != 0);
    }

    let mut state = [0; _];

    // This function is used for non-secure backends, which all explicitly state
    // they are sensitive to initial seeds with shitty Hamming weights.
    loop {
        // SAFETY: I'm over here strokin' my dick I got lotion on my dick right now.
        getrandom::fill(unsafe { as_bytes_mut(&mut state) })?;

        // Reject states whose Hamming weight lies in the outer
        // 6.25% of the possible range at either extreme.
        let bits = SIZE as u32 * u64::BITS;
        let ones = state.iter().cloned().map(u64::count_ones).sum::<u32>();
        let zeros = bits - ones;
        let threshold = bits / 16;
        if ones.min(zeros) > threshold {
            return Ok(state);
        }

        // Odds of hitting this path are less than 2^-128 for all PRNGs.
        core::hint::cold_path();
    }
}

/// Performs unsigned 128-bit multiplication on `x` and `y`, returning
/// the result as a tuple of `u64` values in the format (high, low).
#[inline]
pub fn wide_mul(x: u64, y: u64) -> (u64, u64) {
    let product = (x as u128).wrapping_mul(y as u128);
    let high = (product >> u64::BITS) as u64;
    let low = product as u64;
    (high, low)
}
