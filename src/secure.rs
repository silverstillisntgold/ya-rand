use crate::rng::*;
use chachacha::{BUF_LEN_U64, ChaCha8Djb, SEED_LEN_U8};
use core::fmt::{self, Debug, Formatter};
use core::mem::MaybeUninit;
use getrandom::fill;

/// A cryptographically secure random number generator.
///
/// The current implementation uses ChaCha with 8 rounds and a 64-bit counter.
/// This allows for 1 ZiB (2<sup>70</sup> bytes) of output before cycling.
/// That's over 147 **quintillion** calls to [`SecureRng::u64`].
pub struct SecureRng {
    buf: [u64; BUF_LEN_U64],
    index: usize,
    internal: ChaCha8Djb,
}

impl Debug for SecureRng {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("all `SecureRng` fields are private")
    }
}

impl SecureYARandGenerator for SecureRng {
    #[inline]
    fn fill_bytes(&mut self, dst: &mut [u8]) {
        // The `chachacha` crate provides a thoroughly tested and
        // extremely fast fill implementation.
        self.internal.fill(dst);
    }
}

impl YARandGenerator for SecureRng {
    #[inline]
    fn try_new() -> Result<Self, getrandom::Error> {
        // We randomize **all** bits of the matrix, even the counter.
        // If used as a cipher this approach is completely braindead,
        // but since this is exclusively for use as a CRNG it's fine.
        #[allow(invalid_value)]
        let mut state = unsafe { MaybeUninit::<[u8; SEED_LEN_U8]>::uninit().assume_init() };
        fill(&mut state)?;
        let mut internal = ChaCha8Djb::from(state);
        let buf = internal.get_block_u64();
        let index = 0;
        Ok(Self {
            buf,
            index,
            internal,
        })
    }

    #[cfg_attr(feature = "inline", inline)]
    fn u64(&mut self) -> u64 {
        // TODO: This is the cold path, occuring only once every
        // 32 calls to `Self::u64`. If there is ever a cold/unlikely
        // intrinsic moved to stable then test the performance impact
        // of applying it to this branch.
        if self.index >= self.buf.len() {
            self.index = 0;
            self.internal.fill_block_u64(&mut self.buf);
        }
        // Bounds check is elided thanks to above code.
        let result = self.buf[self.index];
        self.index += 1;
        result
    }
}
