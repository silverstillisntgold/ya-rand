mod sse2;
mod util;

use core::mem::transmute;

use crate::{SecureYARandGenerator, YARandGenerator};
use util::*;

use sse2::SSE as CurrentMachine;

pub struct SecureRng {
    index: usize,
    buf: [u64; BUF_LEN],
    internal: ChaCha,
}

impl SecureYARandGenerator for SecureRng {
    #[inline(never)]
    fn fill_bytes(&mut self, dst: &mut [u8]) {
        const LEN: usize = size_of::<[u64; BUF_LEN]>();
        dst.chunks_exact_mut(LEN).for_each(|cur| {
            let cur_ref: &mut [u8; LEN] = cur.try_into().unwrap();
            let t: &mut [u64; BUF_LEN] = unsafe { transmute(cur_ref) };
            self.internal.block::<CurrentMachine>(t);
        });
        let chunk = dst.chunks_exact_mut(LEN).into_remainder();
        if chunk.len() != 0 {
            unsafe {
                let mut data = [0; BUF_LEN];
                self.internal.block::<CurrentMachine>(&mut data);
                copy(data.as_ptr().cast(), dst.as_mut_ptr(), dst.len());
            }
        }
    }
}

impl YARandGenerator for SecureRng {
    fn try_new() -> Result<Self, getrandom::Error> {
        let mut dest = [0; ChaCha::SEED_LEN];
        getrandom::fill(&mut dest)?;
        let internal = ChaCha::from(dest);
        let buf = [0; BUF_LEN];
        let index = 0;
        Ok(Self {
            index,
            buf,
            internal,
        })
    }

    #[cfg_attr(feature = "inline", inline)]
    fn u64(&mut self) -> u64 {
        if self.index >= self.buf.len() {
            self.internal.block::<CurrentMachine>(&mut self.buf);
            self.index = 0;
        }
        let result = self.buf[self.index];
        self.index += 1;
        result
    }
}
