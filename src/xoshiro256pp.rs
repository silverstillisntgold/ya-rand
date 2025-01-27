use crate::{
    rng::{Generator, SeedableGenerator},
    util::{seeded_state, seeded_state_secure},
};

/// Rust implementation of the xoshiro256++ PRNG.
/// This generator is fast, high-quality, and small,
/// but not cryptographically secure.
///
/// More information can be found at: <https://prng.di.unimi.it/>.
#[derive(Debug, PartialEq, Eq)]
pub struct Xoshiro256pp {
    state: [u64; 4],
}

impl Default for Xoshiro256pp {
    #[inline]
    fn default() -> Self {
        Self::new_with_seed(0)
    }
}

impl SeedableGenerator for Xoshiro256pp {
    fn new_with_seed(seed: u64) -> Self {
        let state = seeded_state(seed);
        Self { state }
    }
}

impl Generator for Xoshiro256pp {
    fn try_new() -> Result<Self, getrandom::Error> {
        let state = seeded_state_secure()?;
        Ok(Self { state })
    }

    #[cfg_attr(feature = "inline", inline)]
    fn u64(&mut self) -> u64 {
        let result = self.state[0]
            .wrapping_add(self.state[3])
            .rotate_left(23)
            .wrapping_add(self.state[0]);
        let temp = self.state[1] << 17;

        self.state[2] ^= self.state[0];
        self.state[3] ^= self.state[1];
        self.state[1] ^= self.state[2];
        self.state[0] ^= self.state[3];

        self.state[2] ^= temp;
        self.state[3] = self.state[3].rotate_left(45);

        result
    }
}
