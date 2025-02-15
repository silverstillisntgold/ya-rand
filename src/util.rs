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

/// Generates a random `String` with length `len`, using the provided
/// `YARandEncoder` to determine minimum secure length and character set.
/// Returns `None` only when provided `len` is less than what the encoder
/// declares to be safe.
///
/// Originally inspired by golang's addition of [`rand.Text`] in release 1.24,
/// but altered to be encoding/length generic and consistently unbiased for
/// non-trivial bases.
///
/// [`rand.Text`]:
/// https://cs.opensource.google/go/go/+/refs/tags/go1.24.0:src/crypto/rand/text.go
#[cfg(all(feature = "secure", feature = "std"))]
#[inline(always)]
pub fn text<E, T>(rng: &mut T, len: usize) -> Option<std::string::String>
where
    E: crate::YARandEncoder,
    T: crate::SecureYARandGenerator,
{
    match len >= E::MIN_LEN {
        true => Some({
            const BYTE_VALUES: usize = 1 << u8::BITS;
            // SAFETY: u8's are a trivial type and we pwomise to
            // always overwrite all of them UwU.
            let mut data = unsafe {
                std::boxed::Box::new_uninit_slice(len)
                    .assume_init()
                    .into_vec()
            };
            // This branch is evaluated at compile time, so concrete
            // implementations in final binaries will only have the
            // contents of the branch suitable for the encoder used.
            if BYTE_VALUES % E::CHARSET.len() == 0 {
                // Fill vec with random data.
                rng.fill_bytes(&mut data);
                // Directly map each random u8 to a character in the set.
                // Since this branch is only reachable when length is a power
                // of two, the modulo gets optimized out and the whole thing
                // gets vectorized. The assembly for this branch is
                // absolutely beautiful.
                // This approach is extremely efficient, but only produces
                // unbiased random sequences when the length of the current
                // `CHARSET` is divisible by the amount of possible u8 values,
                // which is why we need a fallback approach.
                data.iter_mut().for_each(|d| {
                    let random_value = *d as usize;
                    *d = E::CHARSET[random_value % E::CHARSET.len()];
                });
            } else {
                // Alternative approach that's potentially much slower,
                // but always produces unbiased results.
                data.fill_with(|| *rng.choose(E::CHARSET).unwrap());
            }
            // SAFETY: All provided encoders only use ascii values, and custom
            // encoder implementations agree to do the same when implementing
            // the `YARandEncoder` trait.
            unsafe { std::string::String::from_utf8_unchecked(data) }
        }),
        false => None,
    }
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
