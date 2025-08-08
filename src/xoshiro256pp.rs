use crate::rng::{Generator, SeedableGenerator};
use crate::util;

/// Rust implementation of the xoshiro256++ PRNG.
///
/// This generator is very fast, high-quality, and small,
/// but not cryptographically secure.
///
/// More information can be found at: <https://prng.di.unimi.it/>.
#[derive(Debug, PartialEq, Eq)]
pub struct Xoshiro256pp {
    state: [u64; 4],
}

impl Default for Xoshiro256pp {
    fn default() -> Self {
        Self::new_with_seed(0)
    }
}

impl SeedableGenerator for Xoshiro256pp {
    fn new_with_seed(seed: u64) -> Self {
        let state = util::state_from_seed(seed);
        let mut ret = Self { state };
        let _discard_first = ret.u64();
        ret
    }
}

impl Generator for Xoshiro256pp {
    #[inline]
    fn try_new() -> Result<Self, getrandom::Error> {
        let state = util::state_from_entropy()?;
        Ok(Self { state })
    }

    #[cfg_attr(feature = "inline", inline)]
    fn u64(&mut self) -> u64 {
        let result = self.state[0]
            .wrapping_add(self.state[3])
            .rotate_left(23)
            .wrapping_add(self.state[0]);
        let tmp = self.state[1] << 17;

        self.state[2] ^= self.state[0];
        self.state[3] ^= self.state[1];
        self.state[1] ^= self.state[2];
        self.state[0] ^= self.state[3];

        self.state[2] ^= tmp;
        self.state[3] = self.state[3].rotate_left(45);

        result
    }
}
