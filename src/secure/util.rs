use core::mem::{transmute, MaybeUninit};
pub use core::{ops::Add, ptr::copy};

/// "expand 32-byte k"
pub const ROW_A: [i32; 4] = [0x6170_7865, 0x3320_646e, 0x7962_2d32, 0x6b20_6574];
pub const BUF_LEN: usize = 32;
pub const BUF_LEN_BYTES: usize = BUF_LEN * size_of::<u64>();

pub trait Machine: Add<Output = Self> + Clone {
    const DEPTH: i64;

    fn new(state: ChaCha) -> Self;

    fn double_round(&mut self);

    fn fill_block(self, buf: &mut [u64; BUF_LEN]);
}

#[derive(Clone, Copy)]
pub union Row {
    pub i32x4: [i32; 4],
    pub i64x2: [i64; 2],
}

impl Default for Row {
    #[inline(always)]
    fn default() -> Self {
        Self {
            i32x4: Default::default(),
        }
    }
}

#[derive(Default, Clone)]
pub struct ChaCha {
    // Instead of storing `row_a`, we load it each time
    // before running the ChaCha algorithm.
    pub row_b: Row,
    pub row_c: Row,
    pub row_d: Row,
}

impl From<[u8; Self::SEED_LEN]> for ChaCha {
    #[inline(always)]
    fn from(value: [u8; Self::SEED_LEN]) -> Self {
        let mut result = Self::default();
        let result_ptr = &mut result as *mut Self;
        unsafe {
            copy(value.as_ptr(), result_ptr.cast(), Self::SEED_LEN);
            // Zero the counter
            result.row_d.i64x2[0] = 0;
        }
        result
    }
}

impl ChaCha {
    pub const DOUBLE_ROUNDS: usize = 4;

    pub const SEED_LEN: usize = size_of::<Self>();

    #[inline(never)]
    pub fn block<M: Machine>(&mut self, buf: &mut [u64; BUF_LEN]) {
        let mut state = M::new(self.clone());
        let old_state = state.clone();
        for _ in 0..Self::DOUBLE_ROUNDS {
            state.double_round();
        }
        let result = state + old_state;
        result.fill_block(buf);
        unsafe {
            // Increment 64-bit counter
            self.row_d.i64x2[0] = self.row_d.i64x2[0].wrapping_add(M::DEPTH);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_1() {
        const BLOCK_1: [u8; 64] = [
            0xe2, 0x8a, 0x5f, 0xa4, 0xa6, 0x7f, 0x8c, 0x5d, 0xef, 0xed, 0x3e, 0x6f, 0xb7, 0x30,
            0x34, 0x86, 0xaa, 0x84, 0x27, 0xd3, 0x14, 0x19, 0xa7, 0x29, 0x57, 0x2d, 0x77, 0x79,
            0x53, 0x49, 0x11, 0x20, 0xb6, 0x4a, 0xb8, 0xe7, 0x2b, 0x8d, 0xeb, 0x85, 0xcd, 0x6a,
            0xea, 0x7c, 0xb6, 0x08, 0x9a, 0x10, 0x18, 0x24, 0xbe, 0xeb, 0x08, 0x81, 0x4a, 0x42,
            0x8a, 0xab, 0x1f, 0xa2, 0xc8, 0x16, 0x08, 0x1b,
        ];
        const BLOCK_2: [u8; 64] = [
            0x8a, 0x26, 0xaf, 0x44, 0x8a, 0x1b, 0xa9, 0x06, 0x36, 0x8f, 0xd8, 0xc8, 0x38, 0x31,
            0xc1, 0x8c, 0xec, 0x8c, 0xed, 0x81, 0x1a, 0x02, 0x8e, 0x67, 0x5b, 0x8d, 0x2b, 0xe8,
            0xfc, 0xe0, 0x81, 0x16, 0x5c, 0xea, 0xe9, 0xf1, 0xd1, 0xb7, 0xa9, 0x75, 0x49, 0x77,
            0x49, 0x48, 0x05, 0x69, 0xce, 0xb8, 0x3d, 0xe6, 0xa0, 0xa5, 0x87, 0xd4, 0x98, 0x4f,
            0x19, 0x92, 0x5f, 0x5d, 0x33, 0x8e, 0x43, 0x0d,
        ];

        let mut state = ChaCha::default();
        let mut result = [0; BUF_LEN];
        state.block::<crate::secure::sse2::SSE>(&mut result);
        let blocks: [u8; 256] = unsafe { transmute(result) };

        let b1: [u8; 64] = blocks[..64].try_into().unwrap();
        assert!(BLOCK_1 == b1);

        let b2: [u8; 64] = blocks[64..128].try_into().unwrap();
        assert!(BLOCK_2 == b2);
    }
}
