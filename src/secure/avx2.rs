use super::{Machine, Row};
use core::{
    arch::x86_64::{
        __m256i, _mm256_add_epi32, _mm256_or_si256, _mm256_set1_epi32, _mm256_set_epi32,
        _mm256_slli_epi32, _mm256_srli_epi32, _mm256_xor_si256,
    },
    ops::Add,
};

#[derive(Clone, Copy)]
pub struct AVX2 {
    state: [__m256i; 16],
}

impl Add for AVX2 {
    type Output = Self;

    #[inline(always)]
    fn add(mut self, rhs: Self) -> Self::Output {
        unsafe {
            for i in 0..self.state.len() {
                self.state[i] = _mm256_add_epi32(self.state[i], rhs.state[i])
            }
        }
        self
    }
}

macro_rules! shift_left {
    ($value:expr, $LEFT_SHIFT:expr) => {{
        const RIGHT_SHIFT: i32 = i32::BITS as i32 - $LEFT_SHIFT;
        let left_shift = _mm256_slli_epi32::<$LEFT_SHIFT>($value);
        let right_shift = _mm256_srli_epi32::<RIGHT_SHIFT>($value);
        _mm256_or_si256(left_shift, right_shift)
    }};
}

impl AVX2 {
    #[inline(always)]
    fn quarter_round(&mut self, a: usize, b: usize, c: usize, d: usize) {
        unsafe {
            self.state[a] = _mm256_add_epi32(self.state[a], self.state[b]);
            self.state[d] = _mm256_xor_si256(self.state[a], self.state[d]);
            self.state[d] = shift_left!(self.state[d], 16);

            self.state[c] = _mm256_add_epi32(self.state[c], self.state[d]);
            self.state[b] = _mm256_xor_si256(self.state[b], self.state[c]);
            self.state[b] = shift_left!(self.state[b], 12);

            self.state[a] = _mm256_add_epi32(self.state[a], self.state[b]);
            self.state[d] = _mm256_xor_si256(self.state[a], self.state[d]);
            self.state[d] = shift_left!(self.state[d], 8);

            self.state[c] = _mm256_add_epi32(self.state[c], self.state[d]);
            self.state[b] = _mm256_xor_si256(self.state[b], self.state[c]);
            self.state[b] = shift_left!(self.state[b], 7);
        }
    }
}

impl Machine for AVX2 {
    const DEPTH: i64 = size_of::<Self>() as i64 / 64;

    type Output = [u64; size_of::<Self>() / size_of::<u64>()];

    #[inline(always)]
    fn new(state: super::ChaCha) -> Self {
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
        let p5 = increment_counter(base_counter, 4);
        let p6 = increment_counter(base_counter, 5);
        let p7 = increment_counter(base_counter, 6);
        let p8 = increment_counter(base_counter, 7);

        let state = unsafe {
            [
                // Row a
                _mm256_set1_epi32(super::ROW_A[0]),
                _mm256_set1_epi32(super::ROW_A[1]),
                _mm256_set1_epi32(super::ROW_A[2]),
                _mm256_set1_epi32(super::ROW_A[3]),
                // Row b
                _mm256_set1_epi32(state.row_b.i32x4[0]),
                _mm256_set1_epi32(state.row_b.i32x4[1]),
                _mm256_set1_epi32(state.row_b.i32x4[2]),
                _mm256_set1_epi32(state.row_b.i32x4[3]),
                // Row c
                _mm256_set1_epi32(state.row_c.i32x4[0]),
                _mm256_set1_epi32(state.row_c.i32x4[1]),
                _mm256_set1_epi32(state.row_c.i32x4[2]),
                _mm256_set1_epi32(state.row_c.i32x4[3]),
                // Row d
                _mm256_set_epi32(p1[0], p2[0], p3[0], p4[0], p5[0], p6[0], p7[0], p8[0]),
                _mm256_set_epi32(p1[1], p2[1], p3[1], p4[1], p5[1], p6[1], p7[1], p8[1]),
                _mm256_set_epi32(p1[2], p2[2], p3[2], p4[2], p5[2], p6[2], p7[2], p8[2]),
                _mm256_set_epi32(p1[3], p2[3], p3[3], p4[3], p5[3], p6[3], p7[3], p8[3]),
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
