use super::{ChaCha, Machine};
#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;
use core::ops::Add;

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
        let base_counter = unsafe { state.row_d.i64x2[0] };
        let level_1 = base_counter;
        let p1: [i32; 2] = unsafe { core::mem::transmute(level_1) };
        let level_2 = base_counter.wrapping_add(1);
        let p2: [i32; 2] = unsafe { core::mem::transmute(level_2) };
        let level_3 = base_counter.wrapping_add(2);
        let p3: [i32; 2] = unsafe { core::mem::transmute(level_3) };
        let level_4 = base_counter.wrapping_add(3);
        let p4: [i32; 2] = unsafe { core::mem::transmute(level_4) };

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
                _mm_setr_epi32(p1[0], p2[0], p3[0], p4[0]),
                _mm_setr_epi32(p1[1], p2[1], p3[1], p4[1]),
                _mm_set1_epi32(state.row_d.i32x4[2]),
                _mm_set1_epi32(state.row_d.i32x4[3]),
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
