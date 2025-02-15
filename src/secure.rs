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

// This is a manual implementation of ZeroizeOnDrop from the zeroize trait.
//
// https://github.com/RustCrypto/utils/blob/zeroize-v1.8.1/zeroize/src/lib.rs#L773
// https://github.com/RustCrypto/utils/blob/zeroize-v1.8.1/zeroize/src/lib.rs#L754
impl Drop for SecureRng {
    fn drop(&mut self) {
        let self_ptr = (self as *mut Self).cast::<u8>();
        for i in 0..size_of::<SecureRng>() {
            // SAFETY: Trust me bro (see the first link).
            unsafe {
                write_volatile(self_ptr.add(i), 0);
            }
        }
        atomic::compiler_fence(atomic::Ordering::SeqCst);
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
        const SEED_LEN: usize = 32;
        const STREAM_LEN: usize = 12;
        // Using a combined array so we only need a single syscall.
        let mut data = [0; SEED_LEN + STREAM_LEN];
        getrandom::fill(&mut data)?;
        // Both of these unwraps are removed during compilation.
        let seed: [u8; SEED_LEN] = data[..SEED_LEN].try_into().unwrap();
        let stream: [u8; STREAM_LEN] = data[SEED_LEN..].try_into().unwrap();
        let mut internal = ChaCha8Rng::from_seed(seed);
        internal.set_stream(stream);
        Ok(Self { internal })
    }

    #[cfg_attr(feature = "inline", inline)]
    fn u64(&mut self) -> u64 {
        self.internal.next_u64()
    }
}
