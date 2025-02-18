use crate::rng::{SecureYARandGenerator, YARandGenerator};
use chacha20::{
    rand_core::{RngCore, SeedableRng},
    ChaCha8Rng,
};

/// A cryptographically secure random number generator.
///
/// The current implementation is ChaCha with 8 rounds,
/// as supplied by the [`chacha20`] crate.
#[derive(Debug)]
pub struct SecureRng {
    internal: ChaCha8Rng,
}

impl SecureYARandGenerator for SecureRng {
    #[inline(never)]
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.internal.fill_bytes(dest);
    }
}

impl YARandGenerator for SecureRng {
    fn try_new() -> Result<Self, getrandom::Error> {
        const SEED_LEN: usize = 32;
        const STREAM_LEN: usize = 12;
        // Using a combined array so we only need a single syscall.
        let mut data = [0; SEED_LEN + STREAM_LEN];
        getrandom::fill(&mut data)?;
        // Both of these unwraps get optimized out.
        let seed: [u8; SEED_LEN] = data[..SEED_LEN].try_into().unwrap();
        let stream: [u8; STREAM_LEN] = data[SEED_LEN..].try_into().unwrap();
        let mut internal = ChaCha8Rng::from_seed(seed);
        // Randomizing the stream number further decreases the
        // already-low odds of two instances colliding.
        internal.set_stream(stream);
        Ok(Self { internal })
    }

    #[cfg_attr(feature = "inline", inline)]
    fn u64(&mut self) -> u64 {
        self.internal.next_u64()
    }
}
