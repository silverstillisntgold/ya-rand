use crate::{
    rng::{EntropyGenerator, Generator, SecureGenerator},
    util::{RngResult, ShadowGenerator},
};
use rand_chacha::{
    rand_core::{RngCore, SeedableRng},
    ChaCha8Rng,
};

/// A cryptographically secure random number generator.
///
/// The current implementation is ChaCha with 8 rounds,
/// as supplied by the [`rand_chacha`] crate.
#[derive(Debug)]
pub struct SecureRng {
    internal: ChaCha8Rng,
}

impl Iterator for SecureRng {
    type Item = RngResult;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let result = self.u64_as_result();
        Some(result)
    }
}

impl SecureGenerator for SecureRng {
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.internal.fill_bytes(dest);
    }
}

impl EntropyGenerator for SecureRng {
    fn try_new() -> Result<Self, getrandom::Error> {
        let mut seed = <ChaCha8Rng as SeedableRng>::Seed::default();
        getrandom::fill(&mut seed)?;
        let internal = ChaCha8Rng::from_seed(seed);
        Ok(Self { internal })
    }
}

impl Generator for SecureRng {
    #[cfg_attr(feature = "inline", inline)]
    fn u64(&mut self) -> u64 {
        self.internal.next_u64()
    }
}
