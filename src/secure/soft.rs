use super::{
    util::{DEPTH, ROW_A},
    ChaCha, Machine, BUF_LEN,
};
use core::{mem::transmute, ops::Add};

/// Holds 4 independent instances of reference chacha
/// matrix's. Structuring it like this allows the compiler
/// to auto-vectorize the soft implementation on archs which
/// support SIMD operations but don't have an explicit implementation.
/// Will still be slower than explicit instric usage since those
/// have some micro-optimizations but it's still pretty good.
#[derive(Clone)]
pub struct Matrix {
    state: [[i32; 16]; DEPTH],
}

impl Add for Matrix {
    type Output = Self;

    #[inline(always)]
    fn add(mut self, rhs: Self) -> Self::Output {
        for i in 0..self.state.len() {
            for j in 0..self.state[i].len() {
                self.state[i][j] = self.state[i][j].wrapping_add(rhs.state[i][j]);
            }
        }
        self
    }
}

impl Matrix {
    #[inline(always)]
    fn quarter_round(&mut self, a: usize, b: usize, c: usize, d: usize) {
        for matrix in self.state.iter_mut() {
            matrix[a] = matrix[a].wrapping_add(matrix[b]);
            matrix[d] ^= matrix[a];
            matrix[d] = matrix[d].rotate_left(16);

            matrix[c] = matrix[c].wrapping_add(matrix[d]);
            matrix[b] ^= matrix[c];
            matrix[b] = matrix[b].rotate_left(12);

            matrix[a] = matrix[a].wrapping_add(matrix[b]);
            matrix[d] ^= matrix[a];
            matrix[d] = matrix[d].rotate_left(8);

            matrix[c] = matrix[c].wrapping_add(matrix[d]);
            matrix[b] ^= matrix[c];
            matrix[b] = matrix[b].rotate_left(7);
        }
    }
}

impl Machine for Matrix {
    #[inline(always)]
    fn new(state: &ChaCha<Self>) -> Self {
        unsafe {
            // Building an array of `Row`s makes it easy to increment the counters.
            let mut chacha = [[transmute(ROW_A), state.row_b, state.row_c, state.row_d]; DEPTH];
            chacha[1][3].u64x2[0] = chacha[1][3].u64x2[0].wrapping_add(1);
            chacha[2][3].u64x2[0] = chacha[2][3].u64x2[0].wrapping_add(2);
            chacha[3][3].u64x2[0] = chacha[3][3].u64x2[0].wrapping_add(3);
            Self {
                state: transmute(chacha),
            }
        }
    }

    #[inline(always)]
    fn double_round(&mut self) {
        // Column (even) rounds
        self.quarter_round(0, 4, 8, 12);
        self.quarter_round(1, 5, 9, 13);
        self.quarter_round(2, 6, 10, 14);
        self.quarter_round(3, 7, 11, 15);

        // Diagonal (odd) rounds
        self.quarter_round(0, 5, 10, 15);
        self.quarter_round(1, 6, 11, 12);
        self.quarter_round(2, 7, 8, 13);
        self.quarter_round(3, 4, 9, 14);
    }

    #[inline(always)]
    fn fill_block(self, buf: &mut [u64; BUF_LEN]) {
        unsafe {
            *buf = transmute(self);
        }
    }
}
