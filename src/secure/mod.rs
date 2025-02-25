mod soft;
mod sse2;
mod util;

use crate::{SecureYARandGenerator, YARandGenerator};
use core::mem::{transmute, MaybeUninit};
use util::*;

use soft::Matrix as CurrentMachine;

pub struct SecureRng {
    index: usize,
    buf: [u64; BUF_LEN],
    internal: ChaCha<CurrentMachine>,
}

impl SecureYARandGenerator for SecureRng {
    #[inline(never)]
    fn fill_bytes(&mut self, dst: &mut [u8]) {
        const LEN: usize = size_of::<[u64; BUF_LEN]>();
        dst.chunks_exact_mut(LEN).for_each(|cur| {
            let cur_ref: &mut [u8; LEN] = cur.try_into().unwrap();
            let temp: &mut [u64; BUF_LEN] = unsafe { transmute(cur_ref) };
            self.internal.block(temp);
        });
        let chunk = dst.chunks_exact_mut(LEN).into_remainder();
        if chunk.len() != 0 {
            unsafe {
                #[allow(invalid_value)]
                let mut data = MaybeUninit::uninit().assume_init();
                self.internal.block(&mut data);
                copy(data.as_ptr().cast(), dst.as_mut_ptr(), dst.len());
            }
        }
    }
}

impl YARandGenerator for SecureRng {
    fn try_new() -> Result<Self, getrandom::Error> {
        let mut dest = [0; CHACHA_SEED_LEN];
        getrandom::fill(&mut dest)?;
        let mut result = SecureRng {
            index: 0,
            #[allow(invalid_value)]
            buf: unsafe { MaybeUninit::uninit().assume_init() },
            internal: ChaCha::from(dest),
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
