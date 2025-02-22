use super::{ChaCha, Machine, Row};
use core::{
    arch::x86_64::{
        __m128i, _mm_add_epi32, _mm_or_si128, _mm_set1_epi32, _mm_set_epi32, _mm_slli_epi32,
        _mm_srli_epi32, _mm_xor_si128,
    },
    ops::Add,
};

#[derive(Clone, Copy)]
pub struct SSE {
    state: [__m128i; 16],
}

impl Add for SSE {
    type Output = Self;

    #[inline(always)]
    fn add(mut self, rhs: Self) -> Self::Output {
        unsafe {
            for i in 0..self.state.len() {
                self.state[i] = _mm_add_epi32(self.state[i], rhs.state[i])
            }
        }
        self
    }
}

macro_rules! rotate_left {
    ($value:expr, $LEFT_SHIFT:expr) => {{
        const RIGHT_SHIFT: i32 = i32::BITS as i32 - $LEFT_SHIFT;
        let left_shift = _mm_slli_epi32::<$LEFT_SHIFT>($value);
        let right_shift = _mm_srli_epi32::<RIGHT_SHIFT>($value);
        _mm_or_si128(left_shift, right_shift)
    }};
}

impl SSE {
    #[inline(always)]
    fn quarter_round(&mut self, a: usize, b: usize, c: usize, d: usize) {
        unsafe {
            self.state[a] = _mm_add_epi32(self.state[a], self.state[b]);
            self.state[d] = _mm_xor_si128(self.state[a], self.state[d]);
            self.state[d] = rotate_left!(self.state[d], 16);

            self.state[c] = _mm_add_epi32(self.state[c], self.state[d]);
            self.state[b] = _mm_xor_si128(self.state[b], self.state[c]);
            self.state[b] = rotate_left!(self.state[b], 12);

            self.state[a] = _mm_add_epi32(self.state[a], self.state[b]);
            self.state[d] = _mm_xor_si128(self.state[a], self.state[d]);
            self.state[d] = rotate_left!(self.state[d], 8);

            self.state[c] = _mm_add_epi32(self.state[c], self.state[d]);
            self.state[b] = _mm_xor_si128(self.state[b], self.state[c]);
            self.state[b] = rotate_left!(self.state[b], 7);
        }
    }
}

impl Machine for SSE {
    const DEPTH: i64 = size_of::<Self>() as i64 / 64;

    type Output = [u64; size_of::<Self>() / size_of::<u64>()];

    #[inline(always)]
    fn new(state: ChaCha) -> Self {
        let increment_counter = |mut row: Row, incr: i64| -> [i32; 4] {
            unsafe {
                row.i64x2[0] = row.i64x2[0].wrapping_add(incr);
            }
            unsafe { row.i32x4 }
        };

        let base_counter = state.row_d;
        let p1 = increment_counter(base_counter, 0);
        let p2 = increment_counter(base_counter, 1);
        let p3 = increment_counter(base_counter, 2);
        let p4 = increment_counter(base_counter, 3);

        let state = unsafe {
            [
                // Row a
                _mm_set1_epi32(super::ROW_A[0]),
                _mm_set1_epi32(super::ROW_A[1]),
                _mm_set1_epi32(super::ROW_A[2]),
                _mm_set1_epi32(super::ROW_A[3]),
                // Row b
                _mm_set1_epi32(state.row_b.i32x4[0]),
                _mm_set1_epi32(state.row_b.i32x4[1]),
                _mm_set1_epi32(state.row_b.i32x4[2]),
                _mm_set1_epi32(state.row_b.i32x4[3]),
                // Row c
                _mm_set1_epi32(state.row_c.i32x4[0]),
                _mm_set1_epi32(state.row_c.i32x4[1]),
                _mm_set1_epi32(state.row_c.i32x4[2]),
                _mm_set1_epi32(state.row_c.i32x4[3]),
                // Row d
                _mm_set_epi32(p1[0], p2[0], p3[0], p4[0]),
                _mm_set_epi32(p1[1], p2[1], p3[1], p4[1]),
                _mm_set_epi32(p1[2], p2[2], p3[2], p4[2]),
                _mm_set_epi32(p1[3], p2[3], p3[3], p4[3]),
            ]
        };
        Self { state }
    }

    #[inline(always)]
    fn double_round(&mut self) {
        // Even (column) rounds
        self.quarter_round(0, 4, 8, 12);
        self.quarter_round(1, 5, 9, 13);
        self.quarter_round(2, 6, 10, 14);
        self.quarter_round(3, 7, 11, 15);

        // Odd (diagonal) rounds
        self.quarter_round(0, 5, 10, 15);
        self.quarter_round(1, 6, 11, 12);
        self.quarter_round(2, 7, 8, 13);
        self.quarter_round(3, 4, 9, 14);
    }

    #[inline(always)]
    fn flatten(self) -> <Self as Machine>::Output {
        unsafe { core::mem::transmute(self) }
    }
}
