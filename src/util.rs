/// Creates and returns an array filled with random data from the
/// output of a SplitMix64 PRNG, which is seeded using `seed`.
pub fn new_with_seed<const SIZE: usize>(seed: u64) -> [u64; SIZE] {
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
pub fn try_new<const SIZE: usize>() -> Result<[u64; SIZE], getrandom::Error> {
    let mut state = [0; SIZE];
    // SAFETY: I'm over here strokin' my dick
    // I got lotion on my dick right now.
    let state_bytes = unsafe {
        let data = state.as_mut_ptr().cast();
        let len = state.len() * size_of::<u64>();
        core::slice::from_raw_parts_mut(data, len)
    };
    getrandom::fill(state_bytes)?;
    Ok(state)
}

/// Performs 128-bit multiplication on `x` and `y` and returns
/// the result as a tuple of u64 values (high, low).
#[inline(always)]
pub fn wide_mul(x: u64, y: u64) -> (u64, u64) {
    let product = (x as u128) * (y as u128);
    let high = (product >> u64::BITS) as u64;
    let low = product as u64;
    (high, low)
}

/*pub trait IntoInt<T> {
    fn into_int(self) -> T;
}

#[macro_export]
macro_rules! into_impl {
    ($($t:ty), +) => {
        $(
            impl IntoInt<$t> for u64 {
                fn into_int(self) -> $t {
                    self as $t
                }
            }
        )*
    };
}

into_impl!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, isize);*/

#[cfg(test)]
mod test {
    #[test]
    fn wide_mul() {
        const SHIFT: u32 = 48;
        let x = 1 << SHIFT;
        let y = x;
        let (high, low) = super::wide_mul(x, y);
        assert!(high == 1 << ((SHIFT * 2) - u64::BITS));
        assert!(low == 0);
    }
}
