use core::{
    marker::PhantomData,
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
};
pub use core::{ops::Add, ptr::copy};

/// "expand 32-byte k"
pub const ROW_A: [i32; 4] = [0x6170_7865, 0x3320_646e, 0x7962_2d32, 0x6b20_6574];
pub const BUF_LEN: usize = 32;
pub const DEPTH: usize = 4;

pub const CHACHA_DOUBLE_ROUNDS: usize = 4;
pub const CHACHA_SEED_LEN: usize = size_of::<ChaCha<super::CurrentMachine>>();

pub trait Machine: Add<Output = Self> + Clone {
    fn new(state: &ChaCha<Self>) -> Self;

    fn double_round(&mut self);

    fn fill_block(self, buf: &mut [u64; BUF_LEN]);
}

#[derive(Clone, Copy)]
pub struct RawMatrix {
    matrix: [i32; 16],
}

impl Deref for RawMatrix {
    type Target = [i32; 16];

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.matrix
    }
}

impl DerefMut for RawMatrix {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.matrix
    }
}

#[allow(unused)]
#[derive(Clone, Copy)]
pub struct ChaChaFull {
    pub row_a: Row,
    pub row_b: Row,
    pub row_c: Row,
    pub row_d: Row,
}

impl ChaChaFull {
    #[inline(always)]
    pub fn new<M>(chacha: &ChaCha<M>) -> Self {
        Self {
            row_a: Row { i32x4: ROW_A },
            row_b: chacha.row_b,
            row_c: chacha.row_c,
            row_d: chacha.row_d,
        }
    }
}

#[derive(Clone, Copy)]
pub union Row {
    pub i32x4: [i32; 4],
    pub i64x2: [i64; 2],
}

impl Default for Row {
    #[inline(always)]
    fn default() -> Self {
        unsafe { MaybeUninit::zeroed().assume_init() }
    }
}

#[derive(Clone)]
pub struct ChaCha<M> {
    pub row_b: Row,
    pub row_c: Row,
    pub row_d: Row,
    pd: PhantomData<M>,
}

impl<M> Default for ChaCha<M> {
    #[inline(always)]
    fn default() -> Self {
        unsafe { MaybeUninit::zeroed().assume_init() }
    }
}

impl<M> From<[u8; CHACHA_SEED_LEN]> for ChaCha<M> {
    #[inline(always)]
    fn from(value: [u8; CHACHA_SEED_LEN]) -> Self {
        let mut result = Self::default();
        unsafe {
            let result_ptr = &mut result as *mut Self;
            copy(value.as_ptr(), result_ptr.cast(), CHACHA_SEED_LEN);
        }
        result
    }
}

impl<M: Machine> ChaCha<M> {
    #[inline(never)]
    pub fn block(&mut self, buf: &mut [u64; BUF_LEN]) {
        let mut state = M::new(&self);
        let old_state = state.clone();
        unsafe {
            // Increment 64-bit counter
            self.row_d.i64x2[0] = self.row_d.i64x2[0].wrapping_add(DEPTH as i64);
        }
        for _ in 0..CHACHA_DOUBLE_ROUNDS {
            state.double_round();
        }
        let result = state + old_state;
        result.fill_block(buf);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::secure::*;
    use core::mem::transmute;

    #[test]
    fn chacha_sse2_impl() {
        let mut state = ChaCha::<sse2::Matrix>::default();
        let mut result = [0; BUF_LEN];
        state.block(&mut result);
        verify(result);
    }

    #[test]
    fn chacha_soft_impl() {
        let mut state = ChaCha::<soft::Matrix>::default();
        let mut result = [0; BUF_LEN];
        state.block(&mut result);
        verify(result);
    }

    fn verify(stream: [u64; BUF_LEN]) {
        const KEYSTREAM_BLOCK_0: [u8; 64] = [
            0x3e, 0x00, 0xef, 0x2f, 0x89, 0x5f, 0x40, 0xd6, 0x7f, 0x5b, 0xb8, 0xe8, 0x1f, 0x09,
            0xa5, 0xa1, 0x2c, 0x84, 0x0e, 0xc3, 0xce, 0x9a, 0x7f, 0x3b, 0x18, 0x1b, 0xe1, 0x88,
            0xef, 0x71, 0x1a, 0x1e, 0x98, 0x4c, 0xe1, 0x72, 0xb9, 0x21, 0x6f, 0x41, 0x9f, 0x44,
            0x53, 0x67, 0x45, 0x6d, 0x56, 0x19, 0x31, 0x4a, 0x42, 0xa3, 0xda, 0x86, 0xb0, 0x01,
            0x38, 0x7b, 0xfd, 0xb8, 0x0e, 0x0c, 0xfe, 0x42,
        ];

        const KEYSTREAM_BLOCK_1: [u8; 64] = [
            0xd2, 0xae, 0xfa, 0x0d, 0xea, 0xa5, 0xc1, 0x51, 0xbf, 0x0a, 0xdb, 0x6c, 0x01, 0xf2,
            0xa5, 0xad, 0xc0, 0xfd, 0x58, 0x12, 0x59, 0xf9, 0xa2, 0xaa, 0xdc, 0xf2, 0x0f, 0x8f,
            0xd5, 0x66, 0xa2, 0x6b, 0x50, 0x32, 0xec, 0x38, 0xbb, 0xc5, 0xda, 0x98, 0xee, 0x0c,
            0x6f, 0x56, 0x8b, 0x87, 0x2a, 0x65, 0xa0, 0x8a, 0xbf, 0x25, 0x1d, 0xeb, 0x21, 0xbb,
            0x4b, 0x56, 0xe5, 0xd8, 0x82, 0x1e, 0x68, 0xaa,
        ];

        const BYTE_LEN: usize = BUF_LEN * size_of::<u64>();
        let stream_as_bytes: [u8; BYTE_LEN] = unsafe { transmute(stream) };
        let block_0: [u8; 64] = stream_as_bytes[..64].try_into().unwrap();
        let block_1: [u8; 64] = stream_as_bytes[64..128].try_into().unwrap();

        assert!(block_0 == KEYSTREAM_BLOCK_0);
        assert!(block_1 == KEYSTREAM_BLOCK_1);
    }
}
