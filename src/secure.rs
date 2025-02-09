use crate::rng::{SecureYARandGenerator, YARandGenerator};
use chacha20::{
    rand_core::{RngCore, SeedableRng},
    ChaCha8Rng,
};
use core::{mem::size_of, ptr::write};

/// A cryptographically secure random number generator.
///
/// The current implementation is ChaCha with 8 rounds,
/// as supplied by the [`chacha20`] crate.
#[derive(Debug)]
pub struct SecureRng {
    internal: ChaCha8Rng,
}

impl Drop for SecureRng {
    #[inline(never)]
    fn drop(&mut self) {
        let self_ptr = (self as *mut Self).cast::<u8>();
        for i in 0..size_of::<SecureRng>() {
            unsafe {
                // Zeroize uses `write_volatile` because it needs to
                // be generic and might be used for zeroing any kind of
                // memory. Since we know our type is just groups of integer
                // data, we prefer `write`. This allows the compiler to
                // vectorize zeroing the data of `internal`. Disallowing
                // inlining should prevent rustc from trying to optimize
                // this method out.
                write(self_ptr.add(i), 0);
            }
        }
    }
}

impl SecureYARandGenerator for SecureRng {
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
