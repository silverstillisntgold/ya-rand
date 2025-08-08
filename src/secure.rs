use crate::rng::{Generator, SecureGenerator};
use chachacha::{BUF_LEN_U64, ChaCha8Djb, SEED_LEN_U8};
use core::fmt;
use core::mem::MaybeUninit;

/// A cryptographically secure random number generator.
///
/// The current implementation uses ChaCha with 8 rounds and a 64-bit counter.
/// This allows for 1 ZiB (2<sup>70</sup> bytes) of output before repeating.
/// That's over 147 **quintillion** calls to [`SecureRng::u64`].
pub struct SecureRng {
    buf: [u64; BUF_LEN_U64],
    index: usize,
    internal: ChaCha8Djb,
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
        self.internal.fill(dst);
    }
}

impl Generator for SecureRng {
    #[inline]
    fn try_new() -> Result<Self, getrandom::Error> {
        // We want to randomize **all** bits of the matrix, even the counter.
        // They're all going to be overwritten so no need to initialize them.
        #[allow(invalid_value)]
        let mut state = unsafe { MaybeUninit::<[u8; SEED_LEN_U8]>::uninit().assume_init() };
        getrandom::fill(&mut state)?;
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
        // TODO: Use the `unlikely` hint when it comes to stable.
        if self.index >= self.buf.len() {
            self.index = 0;
            self.internal.fill_block_u64(&mut self.buf);
        }
        // SAFETY: We've just guaranteed that `self.index` will be
        // in bounds in the above if-statement.
        let ret = unsafe { *self.buf.get_unchecked(self.index) };
        self.index += 1;
        ret
    }
}
