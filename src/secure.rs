#![allow(invalid_value)]

use crate::{SecureYARandGenerator, YARandGenerator};
use chachacha::ChaCha8Djb;
use core::mem::MaybeUninit;

const CHACHA_OUTPUT: usize = 256;
const CHACHA_OUTPUT_U64: usize = CHACHA_OUTPUT / size_of::<u64>();
const CHACHA_SEED_LEN: usize = 48;

/// A cryptographically secure random number generator.
///
/// The current implementation is ChaCha with 8 rounds.
pub struct SecureRng {
    index: usize,
    buf: [u64; CHACHA_OUTPUT_U64],
    internal: ChaCha8Djb,
}

impl SecureYARandGenerator for SecureRng {
    #[inline]
    fn fill_bytes(&mut self, dst: &mut [u8]) {
        self.internal.fill(dst);
    }
}

impl YARandGenerator for SecureRng {
    fn try_new() -> Result<Self, getrandom::Error> {
        // We randomize **all** bits of the matrix, even the counter.
        // If used in a cipher this approach is completely braindead,
        // but since this is exclusively for use in a CRNG it's fine.
        let mut dest = unsafe { MaybeUninit::<[u8; CHACHA_SEED_LEN]>::uninit().assume_init() };
        getrandom::fill(&mut dest)?;
        let mut result = SecureRng {
            index: 0,
            buf: unsafe { MaybeUninit::uninit().assume_init() },
            internal: dest.into(),
        };
        result.internal.fill_block_u64(&mut result.buf);
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
