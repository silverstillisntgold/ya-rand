mod avx2;
//mod neon;
mod soft;
mod sse2;
mod util;

use crate::{SecureYARandGenerator, YARandGenerator};
use cfg_if::cfg_if;
use core::{
    mem::{transmute, MaybeUninit},
    ptr::copy,
};
use util::*;

cfg_if! {
    if #[cfg(any(target_arch = "x86_64", target_arch = "x86"))] {
        cfg_if! {
            if #[cfg(target_feature = "avx2")] {
                use avx2::Matrix;
            } else if #[cfg(target_feature = "sse2")] {
                use sse2::Matrix;
            } else {
                use soft::Matrix;
            }
        }
    } else if #[cfg(target_feature = "neon")] {
        // TODO: neon implementation
        use soft::Matrix;
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
    #[allow(invalid_value)]
    #[inline(never)]
    fn fill_bytes(&mut self, dst: &mut [u8]) {
        const LEN: usize = size_of::<[u64; BUF_LEN]>();
        dst.chunks_exact_mut(LEN).for_each(|chunk| {
            let chunk_ref: &mut [u8; LEN] = chunk.try_into().unwrap();
            let chunk_reinterpreted: &mut [u64; BUF_LEN] = unsafe { transmute(chunk_ref) };
            self.internal.block(chunk_reinterpreted);
        });
        let chunk = dst.chunks_exact_mut(LEN).into_remainder();
        if chunk.len() != 0 {
            unsafe {
                let mut data = MaybeUninit::uninit().assume_init();
                self.internal.block(&mut data);
                copy(data.as_ptr().cast(), dst.as_mut_ptr(), dst.len());
            }
        }
    }
}

impl YARandGenerator for SecureRng {
    #[allow(invalid_value)]
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
