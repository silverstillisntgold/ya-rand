use crate::rng::{SecureYARandGenerator, YARandGenerator};
use chacha20::{
    rand_core::{RngCore, SeedableRng},
    ChaCha8Rng,
};
use core::{mem::size_of, ptr::write_volatile, sync::atomic};

/// A cryptographically secure random number generator.
///
/// The current implementation is ChaCha with 8 rounds,
/// as supplied by the [`chacha20`] crate.
#[derive(Debug)]
pub struct SecureRng {
    internal: ChaCha8Rng,
}

impl Drop for SecureRng {
    // This approach comes from the zeroize crate.
    fn drop(&mut self) {
        let self_ptr = (self as *mut Self).cast::<u8>();
        for i in 0..size_of::<SecureRng>() {
            unsafe {
                write_volatile(self_ptr.add(i), 0);
            }
        }
        atomic::fence(atomic::Ordering::SeqCst);
    }
}

impl SecureYARandGenerator for SecureRng {
    #[inline(never)]
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.internal.fill_bytes(dest);
    }
}

impl YARandGenerator for SecureRng {
    fn try_new() -> Result<Self, getrandom::Error> {
        let mut seed = <ChaCha8Rng as SeedableRng>::Seed::default();
        getrandom::fill(&mut seed)?;
        let internal = ChaCha8Rng::from_seed(seed);
        Ok(Self { internal })
    }

    #[cfg_attr(feature = "inline", inline)]
    fn u64(&mut self) -> u64 {
        self.internal.next_u64()
    }
}
