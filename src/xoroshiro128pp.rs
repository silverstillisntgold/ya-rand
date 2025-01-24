use crate::{
    rng::{EntropyGenerator, Generator, SeedableGenerator},
    util::{self, RngResult, ShadowGenerator},
};

/// Rust implementation of the xoroshiro128++ PRNG.
/// This generator is fast, high-quality, and small,
/// but not cryptographically secure.
///
/// More information can be found at: https://prng.di.unimi.it/.
#[derive(Debug, PartialEq, Eq)]
pub struct Xoroshiro128pp {
    state: [u64; 2],
}

impl Default for Xoroshiro128pp {
    #[inline]
    fn default() -> Self {
        Self::new_with_seed(0)
    }
}

impl Iterator for Xoroshiro128pp {
    type Item = RngResult;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let result = self.u64_as_result();
        Some(result)
    }
}

impl SeedableGenerator for Xoroshiro128pp {
    fn new_with_seed(seed: u64) -> Self {
        let state = util::seeded_state(seed);
        Self { state }
    }
}

impl EntropyGenerator for Xoroshiro128pp {
    fn try_new() -> Result<Self, getrandom::Error> {
        let state = util::seeded_state_secure()?;
        Ok(Self { state })
    }
}

impl Generator for Xoroshiro128pp {
    #[cfg_attr(feature = "inline", inline)]
    fn u64(&mut self) -> u64 {
        let s0 = self.state[0];
        let mut s1 = self.state[1];
        let result = s0.wrapping_add(s1).rotate_left(17).wrapping_add(s0);

        s1 ^= s0;
        self.state[0] = s0.rotate_left(49) ^ s1 ^ (s1 << 21);
        self.state[1] = s1.rotate_left(28);

        result
    }
}
