use crate::rng::{Generator, SecureGenerator};
use chachacha::{BATCH_BYTES, ChaChaDjb};
use core::fmt;

type ChaCha = ChaChaDjb<10>;

union Buffer {
    u8: [u8; BATCH_BYTES],
    u64: [u64; BATCH_BYTES / size_of::<u64>()],
}

impl Default for Buffer {
    #[inline]
    fn default() -> Self {
        Self { u8: [0; _] }
    }
}

/// A cryptographically secure random number generator.
///
/// The current implementation uses ChaCha with 10 rounds and a 64-bit counter.
/// This allows for 1 ZiB (2<sup>70</sup> bytes) of output before repeating.
/// That's over 147 **quintillion** calls to [`SecureRng::u64`].
pub struct SecureRng {
    buffer: Buffer,
    rng: ChaCha,
    index: usize,
}

impl fmt::Debug for SecureRng {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("all `SecureRng` fields are private")
    }
}

impl SecureGenerator for SecureRng {
    #[inline]
    fn fill_bytes(&mut self, dst: &mut [u8]) {
        // The `chachacha` crate provides a thoroughly tested and
        // extremely fast fill implementation.
        self.rng.fill(dst);
    }
}

impl Generator for SecureRng {
    #[inline(never)]
    fn try_new() -> Result<Self, getrandom::Error> {
        // We want to randomize **all** bits of the matrix, even the counter.
        let mut state = [0; _];
        getrandom::fill(&mut state)?;
        let mut buffer = Buffer::default();
        let mut rng = ChaCha::from_bytes(state);
        unsafe {
            rng.fill_exact(&mut buffer.u8);
        }
        let index = 0;
        Ok(Self { buffer, rng, index })
    }

    #[cfg_attr(feature = "inline", inline)]
    fn u64(&mut self) -> u64 {
        unsafe {
            if self.index >= self.buffer.u64.len() {
                // This branch only occurs when `self.buffer.u64` has
                // been exhausted: once every 32 calls.
                core::hint::cold_path();
                self.rng.fill_exact(&mut self.buffer.u8);
                // Needs to be zeroed **after** `self.buffer.u8` is filled.
                self.index = 0;
            }
            // Control flow of the above if-statement ensures
            // this bounds check is optimized out.
            let ret = self.buffer.u64[self.index];
            self.index += 1;
            ret
        }
    }
}
