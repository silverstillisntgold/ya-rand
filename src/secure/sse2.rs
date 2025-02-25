use super::{util::DEPTH, ChaCha, Machine, BUF_LEN, ROW_A};
#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;
use core::{mem::transmute, ops::Add};

#[derive(Clone, Copy)]
pub struct Matrix {
    state: [[__m128i; 4]; DEPTH],
}

impl Add for Matrix {
    type Output = Self;

    #[inline(always)]
    fn add(mut self, rhs: Self) -> Self::Output {
        unsafe {
            for i in 0..self.state.len() {
                for j in 0..self.state[i].len() {
                    self.state[i][j] = _mm_add_epi32(self.state[i][j], rhs.state[i][j]);
                }
            }
            self
        }
    }
}

macro_rules! rotate_left_epi32 {
    ($value:expr, $LEFT_SHIFT:expr) => {{
        const RIGHT_SHIFT: i32 = i32::BITS as i32 - $LEFT_SHIFT;
        let left_shift = _mm_slli_epi32($value, $LEFT_SHIFT);
        let right_shift = _mm_srli_epi32($value, RIGHT_SHIFT);
        _mm_or_si128(left_shift, right_shift)
    }};
}

impl Matrix {
    #[inline(always)]
    fn quarter_round(&mut self) {
        unsafe {
            for [a, b, c, d] in self.state.iter_mut() {
                *a = _mm_add_epi32(*a, *b);
                *d = _mm_xor_si128(*d, *a);
                *d = rotate_left_epi32!(*d, 16);

                *c = _mm_add_epi32(*c, *d);
                *b = _mm_xor_si128(*b, *c);
                *b = rotate_left_epi32!(*b, 12);

                *a = _mm_add_epi32(*a, *b);
                *d = _mm_xor_si128(*d, *a);
                *d = rotate_left_epi32!(*d, 8);

                *c = _mm_add_epi32(*c, *d);
                *b = _mm_xor_si128(*b, *c);
                *b = rotate_left_epi32!(*b, 7);
            }
        }
    }

    #[inline(always)]
    fn make_diagonal(&mut self) {
        unsafe {
            for [a, _, c, d] in self.state.iter_mut() {
                *c = _mm_shuffle_epi32(*c, 0b_00_11_10_01); // _MM_SHUFFLE(0, 3, 2, 1)
                *d = _mm_shuffle_epi32(*d, 0b_01_00_11_10); // _MM_SHUFFLE(1, 0, 3, 2)
                *a = _mm_shuffle_epi32(*a, 0b_10_01_00_11); // _MM_SHUFFLE(2, 1, 0, 3)
            }
        }
    }

    #[inline(always)]
    fn unmake_diagonal(&mut self) {
        unsafe {
            for [a, _, c, d] in self.state.iter_mut() {
                *c = _mm_shuffle_epi32(*c, 0b_10_01_00_11); // _MM_SHUFFLE(2, 1, 0, 3)
                *d = _mm_shuffle_epi32(*d, 0b_01_00_11_10); // _MM_SHUFFLE(1, 0, 3, 2)
                *a = _mm_shuffle_epi32(*a, 0b_00_11_10_01); // _MM_SHUFFLE(0, 3, 2, 1)
            }
        }
    }
}

impl Machine for Matrix {
    #[inline(always)]
    fn new(state: &ChaCha<Self>) -> Self {
        unsafe {
            let row_a = transmute(ROW_A);
            let row_b = transmute(state.row_b);
            let row_c = transmute(state.row_c);
            let row_d = transmute(state.row_d);
            let mut state = Matrix {
                state: [[row_a, row_b, row_c, row_d]; DEPTH],
            };
            state.state[1][3] = _mm_add_epi64(state.state[1][3], _mm_set_epi64x(0, 1));
            state.state[2][3] = _mm_add_epi64(state.state[2][3], _mm_set_epi64x(0, 2));
            state.state[3][3] = _mm_add_epi64(state.state[3][3], _mm_set_epi64x(0, 3));
            state
        }
    }

    #[inline(always)]
    fn double_round(&mut self) {
        self.quarter_round();
        self.make_diagonal();
        self.quarter_round();
        self.unmake_diagonal();
    }

    #[inline(always)]
    fn fill_block(self, buf: &mut [u64; BUF_LEN]) {
        unsafe {
            *buf = transmute(self);
        }
    }
}
