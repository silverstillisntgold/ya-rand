/*!
TODO: Module level documentation explaining approach to vectorized implementations
and optimizations.

https://eprint.iacr.org/2013/759
*/

#![allow(invalid_value)]

mod soft;
mod util;

use crate::{SecureYARandGenerator, YARandGenerator};
use cfg_if::cfg_if;
use core::{
    mem::{transmute, MaybeUninit},
    ptr::copy_nonoverlapping,
};
use util::*;

cfg_if! {
    if #[cfg(any(target_arch = "x86_64", target_arch = "x86"))] {
        mod avx2;
        mod sse2;
        cfg_if! {
            if #[cfg(all(feature = "nightly", target_feature = "avx512f"))] {
                mod avx512;
                use avx512::Matrix;
            } else if #[cfg(target_feature = "avx2")] {
                use avx2::Matrix;
            } else if #[cfg(target_feature = "sse2")] {
                use sse2::Matrix;
            } else {
                use soft::Matrix;
            }
        }
    // NEON on ARM32 is both unsound and gated behind nightly.
    } else if #[cfg(all(target_feature = "neon", any(target_arch = "aarch64", target_arch = "arm64ec")))] {
        mod neon;
        use neon::Matrix;
    } else {
        use soft::Matrix;
    }
}

pub struct SecureRng {
    index: usize,
    buf: [u64; BUF_LEN],
    internal: ChaCha<Matrix>,
}

impl SecureYARandGenerator for SecureRng {
    #[inline(never)]
    fn fill_bytes(&mut self, dst: &mut [u8]) {
        const LEN: usize = size_of::<[u64; BUF_LEN]>();
        dst.chunks_exact_mut(LEN).for_each(|chunk| {
            let chunk_ref: &mut [u8; LEN] = chunk.try_into().unwrap();
            let chunk_reinterpreted: &mut [u64; BUF_LEN] = unsafe { transmute(chunk_ref) };
            self.internal.block(chunk_reinterpreted);
        });
        let remaining_chunk = dst.chunks_exact_mut(LEN).into_remainder();
        if remaining_chunk.len() != 0 {
            unsafe {
                let mut buf = MaybeUninit::uninit().assume_init();
                self.internal.block(&mut buf);
                copy_nonoverlapping(
                    buf.as_ptr().cast(),
                    remaining_chunk.as_mut_ptr(),
                    remaining_chunk.len(),
                );
            }
        }
    }
}

impl YARandGenerator for SecureRng {
    fn try_new() -> Result<Self, getrandom::Error> {
        let mut dest = unsafe { MaybeUninit::<[u8; CHACHA_SEED_LEN]>::uninit().assume_init() };
        getrandom::fill(&mut dest)?;
        let mut result = SecureRng {
            index: 0,
            buf: unsafe { MaybeUninit::uninit().assume_init() },
            internal: dest.into(),
        };
        result.internal.block(&mut result.buf);
        Ok(result)
    }

    #[cfg_attr(feature = "inline", inline)]
    fn u64(&mut self) -> u64 {
        if self.index >= self.buf.len() {
            self.internal.block(&mut self.buf);
            self.index = 0;
        }
        let result = self.buf[self.index];
        self.index += 1;
        result
    }
}
