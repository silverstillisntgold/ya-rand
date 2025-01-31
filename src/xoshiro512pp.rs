use crate::{
    rng::{SeedableYARandGenerator, YARandGenerator},
    util::{state_from_entropy, state_from_seed},
};

/// Rust implementation of the xoshiro512++ PRNG.
/// This generator is fast, high-quality, and small,
/// but not cryptographically secure.
///
/// More information can be found at: <https://prng.di.unimi.it/>.
#[derive(Debug, PartialEq, Eq)]
pub struct Xoshiro512pp {
    state: [u64; 8],
}

impl Default for Xoshiro512pp {
    #[inline]
    fn default() -> Self {
        Self::new_with_seed(0)
    }
}

impl SeedableYARandGenerator for Xoshiro512pp {
    fn new_with_seed(seed: u64) -> Self {
        let state = state_from_seed(seed);
        Self { state }
    }
}

impl YARandGenerator for Xoshiro512pp {
    fn try_new() -> Result<Self, getrandom::Error> {
        let state = state_from_entropy()?;
        Ok(Self { state })
    }

    #[cfg_attr(feature = "inline", inline)]
    fn u64(&mut self) -> u64 {
        let result = self.state[0]
            .wrapping_add(self.state[2])
            .rotate_left(17)
            .wrapping_add(self.state[2]);
        let temp = self.state[1] << 11;

        self.state[2] ^= self.state[0];
        self.state[5] ^= self.state[1];
        self.state[1] ^= self.state[2];
        self.state[7] ^= self.state[3];
        self.state[3] ^= self.state[4];
        self.state[4] ^= self.state[5];
        self.state[0] ^= self.state[6];
        self.state[6] ^= self.state[7];

        self.state[6] ^= temp;
        self.state[7] = self.state[7].rotate_left(21);

        result
    }
}
