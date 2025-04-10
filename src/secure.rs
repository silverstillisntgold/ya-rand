use crate::{SecureYARandGenerator, YARandGenerator};
use chachacha::*;
use core::mem::MaybeUninit;

/// A cryptographically secure random number generator.
///
/// The current implementation is ChaCha with 8 rounds,
/// using the original variation (64-bit counter).
pub struct SecureRng {
    index: usize,
    buf: [u64; BUF_LEN_U64],
    internal: ChaCha8Djb,
}

impl SecureYARandGenerator for SecureRng {
    #[inline]
    fn fill_bytes(&mut self, dst: &mut [u8]) {
        self.internal.fill(dst);
    }
}

impl YARandGenerator for SecureRng {
    #[inline]
    fn try_new() -> Result<Self, getrandom::Error> {
        // We randomize **all** bits of the matrix, even the counter.
        // If used in a cipher this approach is completely braindead,
        // but since this is exclusively for use in a CRNG it's fine.
        #[allow(invalid_value)]
        let mut state = unsafe { MaybeUninit::<[u8; SEED_LEN]>::uninit().assume_init() };
        getrandom::fill(&mut state)?;
        let mut internal = ChaCha8Djb::new(state);
        let buf = internal.get_block_u64();
        let result = SecureRng {
            index: 0,
            buf,
            internal,
        };
        Ok(result)
    }

    #[cfg_attr(feature = "inline", inline)]
    fn u64(&mut self) -> u64 {
        if self.index >= self.buf.len() {
            self.internal.fill_block_u64(&mut self.buf);
            self.index = 0;
        }
        let result = self.buf[self.index];
        self.index += 1;
        result
    }
}
