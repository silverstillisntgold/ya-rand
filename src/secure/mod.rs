pub mod avx2;
pub mod sse;

use crate::{SecureYARandGenerator, YARandGenerator};
use core::ops::Add;
use sse::SSE as CurrentMachine;

pub struct SecureRng {
    index: usize,
    buf: <CurrentMachine as Machine>::Output,
    internal: ChaCha,
}

impl SecureYARandGenerator for SecureRng {
    #[inline(never)]
    fn fill_bytes(&mut self, dst: &mut [u8]) {
        const LEN: usize = size_of::<<CurrentMachine as Machine>::Output>();
        dst.chunks_exact_mut(LEN).for_each(|cur| {
            let cur_ref: &mut [u8; LEN] = cur.try_into().unwrap();
            *cur_ref = unsafe { core::mem::transmute(self.internal.block::<CurrentMachine>()) };
        });
        let chunk = dst.chunks_exact_mut(LEN).into_remainder();
        if chunk.len() != 0 {
            unsafe {
                let data: [u8; LEN] = core::mem::transmute(self.internal.block::<CurrentMachine>());
                core::ptr::copy(data.as_ptr(), dst.as_mut_ptr(), dst.len());
            }
        }
    }
}

impl YARandGenerator for SecureRng {
    fn try_new() -> Result<Self, getrandom::Error> {
        let mut dest = [0; ChaCha::SEED_LEN];
        getrandom::fill(&mut dest)?;
        let mut internal = ChaCha::from(dest);
        let buf = internal.block::<CurrentMachine>();
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
            self.buf = self.internal.block::<CurrentMachine>();
            self.index = 0;
        }
        let result = self.buf[self.index];
        self.index += 1;
        result
    }
}

/// "DanielJBernstein" :)
const ROW_A: [i32; 4] = [0x696e6144, 0x424a6c65, 0x736e7265, 0x6e696574];

#[derive(Clone, Copy)]
union Row {
    pub i32x4: [i32; 4],
    pub i64x2: [i64; 2],
}

impl Default for Row {
    #[inline(always)]
    fn default() -> Self {
        Self {
            i64x2: Default::default(),
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct ChaCha {
    // Instead of storing `row_a`, we load it each time
    // before running the ChaCha algorithm.
    // row_a: Row,
    row_b: Row,
    row_c: Row,
    row_d: Row,
}

impl From<[u8; Self::SEED_LEN]> for ChaCha {
    #[inline(always)]
    fn from(value: [u8; Self::SEED_LEN]) -> Self {
        let mut result = Self::default();
        let result_ptr = &mut result as *mut Self;
        unsafe {
            core::ptr::copy(value.as_ptr(), result_ptr.cast(), Self::SEED_LEN);
            // Zero counter
            result.row_d.i64x2[0] = 0;
        }
        result
    }
}

impl ChaCha {
    pub const DOUBLE_ROUNDS: usize = 4;

    pub const SEED_LEN: usize = size_of::<Self>();

    #[inline(never)]
    pub fn block<M: Machine>(&mut self) -> <M as Machine>::Output {
        let mut state = M::new(*self);
        let old_state = state;
        for _ in 0..Self::DOUBLE_ROUNDS {
            state.double_round();
        }
        unsafe {
            // Increment 64-bit counter
            self.row_d.i64x2[0] = self.row_d.i64x2[0].wrapping_add(M::DEPTH);
        }
        let result = state + old_state;
        result.flatten()
    }
}

pub trait Machine: Add<Output = Self> + Clone + Copy {
    const DEPTH: i64;

    type Output;

    fn new(state: ChaCha) -> Self;

    fn double_round(&mut self);

    fn flatten(self) -> <Self as Machine>::Output;
}
