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

#[cfg(all(feature = "secure", feature = "std"))]
#[inline(always)]
pub fn text<E, T>(rng: &mut T, len: usize) -> Option<std::string::String>
where
    E: crate::encoding::Encoding,
    T: crate::SecureYARandGenerator,
{
    match len >= E::MIN_LEN {
        true => Some({
            let mut data = std::vec![0; len];
            // When this if condition is satisfied, we can use the extremely fast
            // approach of filling `data` with random bytes, then mapping those
            // bytes to our `CHARSET` values directly via modulo. This works because
            // this if clause succeeding guarantees even divisibility, which guarantees
            // even (uniform) distribution. This is evaluated at compile time,
            // and the generated assembly is absolutely beautiful.
            const BYTE_VALUES: usize = 1 << u8::BITS;
            if BYTE_VALUES % E::CHARSET.len() == 0 {
                // Implementation from golang's 1.24 release, but modified to be encoding generic.
                // https://cs.opensource.google/go/go/+/refs/tags/go1.24.0:src/crypto/rand/text.go
                rng.fill_bytes(&mut data);
                for i in 0..data.len() {
                    let val = data[i] as usize;
                    data[i] = E::CHARSET[val % E::CHARSET.len()];
                }
            } else {
                // Alternative approach that remains unbiased, but isn't as fast.
                data.fill_with(|| *rng.choose(E::CHARSET).unwrap());
            }
            // SAFETY: All internal encodings only use ascii values, and custom
            // encoding implementations are expected to do the same.
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
