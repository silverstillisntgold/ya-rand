use super::{util::DEPTH, ChaCha, Machine, BUF_LEN, ROW_A};
use crate::secure::Row;
use core::{mem::transmute, ops::Add};

#[derive(Clone)]
pub struct Soft {
    state: [[i32; 16]; DEPTH],
}

impl Add for Soft {
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

impl Soft {
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

#[allow(unused)]
#[derive(Clone, Copy)]
struct ChaChaTemp {
    row_a: Row,
    row_b: Row,
    row_c: Row,
    row_d: Row,
}

impl Machine for Soft {
    #[inline(always)]
    fn new(state: &ChaCha) -> Self {
        let mut state = [ChaChaTemp {
            row_a: unsafe { transmute(ROW_A) },
            row_b: state.row_b,
            row_c: state.row_c,
            row_d: state.row_d,
        }; DEPTH];
        for i in 0..state.len() {
            unsafe {
                state[i].row_d.i64x2[0] = state[i].row_d.i64x2[0].wrapping_add(i as i64);
            }
        }
        unsafe { transmute(state) }
    }

    #[inline(always)]
    fn double_round(&mut self) {
        self.quarter_round(0, 4, 8, 12);
        self.quarter_round(1, 5, 9, 13);
        self.quarter_round(2, 6, 10, 14);
        self.quarter_round(3, 7, 11, 15);

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
