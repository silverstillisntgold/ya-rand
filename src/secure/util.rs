use core::ops::Add;
use core::{
    marker::PhantomData,
    mem::{transmute, MaybeUninit},
};

/// "expand 32-byte k", the standard constant for ChaCha.
pub const ROW_A: [i32; 4] = [0x6170_7865, 0x3320_646e, 0x7962_2d32, 0x6b20_6574];
/// Normally, ChaCha outputs only 16 `i32`'s, but we process in
/// chunks of 4 and output `u64`'s.
pub const BUF_LEN: usize = 16 * 4 / 2;
/// Since we process in chunks of 4, the counter of the base
/// ChaCha instance needs to be incremented by 4.
pub const DEPTH: usize = 4;

/// 4 double rounds makes this a ChaCha8 implementation.
/// Increasing this would be trivial if ever needed, but the
/// test datastreams would need to be updated as well.
pub const CHACHA_DOUBLE_ROUNDS: usize = 4;
pub const CHACHA_SEED_LEN: usize = size_of::<ChaCha<super::Matrix>>();

/// Defines the interface that concrete implementations need to
/// implement to process the state of a `ChaCha` instance.
///
/// Instances should always process `DEPTH` amount of chacha blocks.
pub trait Machine: Add<Output = Self> + Clone {
    /// Uses the current `ChaCha` state to create a new `Machine`,
    /// which will internally handle it's own counters.
    fn new(state: &ChaCha<Self>) -> Self;

    /// Process a standard double round of the ChaCha algorithm.
    ///
    /// The way that the `Machine` goes about this is completely up
    /// to the implementation, but the result should always be able
    /// to pass all the test.
    fn double_round(&mut self);

    /// Fills `buf` with the output of four processed ChaCha blocks.
    /// It's computationally cheaper to fill a passed-in buffer than
    /// to create and return a new one.
    fn fill_block(self, buf: &mut [u64; BUF_LEN]);
}

/// Wrapper for the data of a `ChaCha` row. In a reference
/// implementation this would just be the `i32x4` field, but having
/// `i64x2` is useful for working with a 64-bit counter and `i8x16`
/// is useful for testing. `i16x8` is included for completeness.
#[allow(unused)]
#[derive(Clone, Copy)]
pub union Row {
    pub i8x16: [i8; 16],
    pub i16x8: [i16; 8],
    pub i32x4: [i32; 4],
    pub i64x2: [i64; 2],
}

/// Long-term storage for the state of a chacha matrix.
/// There is no `row_a` stored since we can just load it in from
/// a static array when creating a new `Machine`.
#[derive(Clone)]
pub struct ChaCha<M> {
    pub row_b: Row,
    pub row_c: Row,
    pub row_d: Row,
    _pd: PhantomData<M>,
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
        // We randomize **all** bits of the matrix, even the counter.
        // If used as a cipher this is a completely braindead approach,
        // but since this is exclusively for use in a CRNG it's fine.
        unsafe { transmute(value) }
    }
}

impl<M: Machine> ChaCha<M> {
    /// Computes 4 blocks of chacha and fills `buf` with the output.
    ///
    /// This is the inline boundary. Everything beneath this should be
    /// marked `#[inline(always)]`, since most (all?) of it should be vector
    /// intrinsics or capable of being optimized into vector instructions.
    #[inline(never)]
    pub fn block(&mut self, buf: &mut [u64; BUF_LEN]) {
        let mut state = M::new(self);
        let old_state = state.clone();
        unsafe {
            // Increment 64-bit counter by `DEPTH`. Since we randomize
            // the counter, it's important that this uses `wrapping_add`,
            // otherwise debug builds might fuck themselves over.
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
    //! All keystreams for these tests come from [here].
    //!
    //! [here]:
    //! https://github.com/secworks/chacha_testvectors/blob/master/src/chacha_testvectors.txt

    use super::*;
    use crate::secure::*;
    use core::mem::transmute;

    #[test]
    fn correct_constant() {
        const EXPECTED: &[u8; 16] = b"expand 32-byte k";
        const ACTUAL: [u8; 16] = unsafe { transmute(ROW_A) };
        assert!(ACTUAL == *EXPECTED);
    }

    #[cfg(target_feature = "neon")]
    #[test]
    fn chacha_neon() {
        chacha_test::<neon::Matrix>();
    }

    #[cfg(target_feature = "avx2")]
    #[test]
    fn chacha_avx2() {
        chacha_test::<avx2::Matrix>();
    }

    #[cfg(target_feature = "sse2")]
    #[test]
    fn chacha_sse2() {
        chacha_test::<sse2::Matrix>();
    }

    #[test]
    fn chacha_soft() {
        chacha_test::<soft::Matrix>();
    }

    fn chacha_test<M: Machine>() {
        let reset = || ChaCha::<M>::default();
        let mut data = [0; BUF_LEN];

        let mut state = reset();
        state.block(&mut data);
        all_bytes_zeroed(data);

        state = reset();
        unsafe {
            state.row_b.i8x16[0] = 1;
        }
        state.block(&mut data);
        first_key_byte_is_1(data);

        state = reset();
        unsafe {
            state.row_d.i8x16[8] = 1;
        }
        state.block(&mut data);
        first_nonce_byte_is_1(data);

        const NOT_BITS: i64 = !0;
        state.row_b.i64x2 = [NOT_BITS, NOT_BITS];
        state.row_c.i64x2 = [NOT_BITS, NOT_BITS];
        // Tests always expect the counter to start at 0.
        state.row_d.i64x2 = [0, NOT_BITS];
        state.block(&mut data);
        all_bytes_set(data);
    }

    fn all_bytes_zeroed(data: [u64; BUF_LEN]) {
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
        let (block_0, block_1) = fetch_blocks(data);
        assert!(block_0 == KEYSTREAM_BLOCK_0);
        assert!(block_1 == KEYSTREAM_BLOCK_1);
    }

    fn first_key_byte_is_1(data: [u64; BUF_LEN]) {
        const KEYSTREAM_BLOCK_0: [u8; 64] = [
            0xcf, 0x5e, 0xe9, 0xa0, 0x49, 0x4a, 0xa9, 0x61, 0x3e, 0x05, 0xd5, 0xed, 0x72, 0x5b,
            0x80, 0x4b, 0x12, 0xf4, 0xa4, 0x65, 0xee, 0x63, 0x5a, 0xcc, 0x3a, 0x31, 0x1d, 0xe8,
            0x74, 0x04, 0x89, 0xea, 0x28, 0x9d, 0x04, 0xf4, 0x3c, 0x75, 0x18, 0xdb, 0x56, 0xeb,
            0x44, 0x33, 0xe4, 0x98, 0xa1, 0x23, 0x8c, 0xd8, 0x46, 0x4d, 0x37, 0x63, 0xdd, 0xbb,
            0x92, 0x22, 0xee, 0x3b, 0xd8, 0xfa, 0xe3, 0xc8,
        ];
        const KEYSTREAM_BLOCK_1: [u8; 64] = [
            0xb4, 0x35, 0x5a, 0x7d, 0x93, 0xdd, 0x88, 0x67, 0x08, 0x9e, 0xe6, 0x43, 0x55, 0x8b,
            0x95, 0x75, 0x4e, 0xfa, 0x2b, 0xd1, 0xa8, 0xa1, 0xe2, 0xd7, 0x5b, 0xcd, 0xb3, 0x20,
            0x15, 0x54, 0x26, 0x38, 0x29, 0x19, 0x41, 0xfe, 0xb4, 0x99, 0x65, 0x58, 0x7c, 0x4f,
            0xdf, 0xe2, 0x19, 0xcf, 0x0e, 0xc1, 0x32, 0xa6, 0xcd, 0x4d, 0xc0, 0x67, 0x39, 0x2e,
            0x67, 0x98, 0x2f, 0xe5, 0x32, 0x78, 0xc0, 0xb4,
        ];
        let (block_0, block_1) = fetch_blocks(data);
        assert!(block_0 == KEYSTREAM_BLOCK_0);
        assert!(block_1 == KEYSTREAM_BLOCK_1);
    }

    fn first_nonce_byte_is_1(data: [u64; BUF_LEN]) {
        const KEYSTREAM_BLOCK_0: [u8; 64] = [
            0x2b, 0x8f, 0x4b, 0xb3, 0x79, 0x83, 0x06, 0xca, 0x51, 0x30, 0xd4, 0x7c, 0x4f, 0x8d,
            0x4e, 0xd1, 0x3a, 0xa0, 0xed, 0xcc, 0xc1, 0xbe, 0x69, 0x42, 0x09, 0x0f, 0xae, 0xec,
            0xa0, 0xd7, 0x59, 0x9b, 0x7f, 0xf0, 0xfe, 0x61, 0x6b, 0xb2, 0x5a, 0xa0, 0x15, 0x3a,
            0xd6, 0xfd, 0xc8, 0x8b, 0x95, 0x49, 0x03, 0xc2, 0x24, 0x26, 0xd4, 0x78, 0xb9, 0x7b,
            0x22, 0xb8, 0xf9, 0xb1, 0xdb, 0x00, 0xcf, 0x06,
        ];
        const KEYSTREAM_BLOCK_1: [u8; 64] = [
            0x47, 0x0b, 0xdf, 0xfb, 0xc4, 0x88, 0xa8, 0xb7, 0xc7, 0x01, 0xeb, 0xf4, 0x06, 0x1d,
            0x75, 0xc5, 0x96, 0x91, 0x86, 0x49, 0x7c, 0x95, 0x36, 0x78, 0x09, 0xaf, 0xa8, 0x0b,
            0xd8, 0x43, 0xb0, 0x40, 0xa7, 0x9a, 0xbc, 0x6e, 0x73, 0xa9, 0x17, 0x57, 0xf1, 0xdb,
            0x73, 0xc8, 0xea, 0xcf, 0xa5, 0x43, 0xb3, 0x8f, 0x28, 0x9d, 0x06, 0x5a, 0xb2, 0xf3,
            0x03, 0x2d, 0x37, 0x7b, 0x8c, 0x37, 0xfe, 0x46,
        ];
        let (block_0, block_1) = fetch_blocks(data);
        assert!(block_0 == KEYSTREAM_BLOCK_0);
        assert!(block_1 == KEYSTREAM_BLOCK_1);
    }

    fn all_bytes_set(data: [u64; BUF_LEN]) {
        const KEYSTREAM_BLOCK_0: [u8; 64] = [
            0xe1, 0x63, 0xbb, 0xf8, 0xc9, 0xa7, 0x39, 0xd1, 0x89, 0x25, 0xee, 0x83, 0x62, 0xda,
            0xd2, 0xcd, 0xc9, 0x73, 0xdf, 0x05, 0x22, 0x5a, 0xfb, 0x2a, 0xa2, 0x63, 0x96, 0xf2,
            0xa9, 0x84, 0x9a, 0x4a, 0x44, 0x5e, 0x05, 0x47, 0xd3, 0x1c, 0x16, 0x23, 0xc5, 0x37,
            0xdf, 0x4b, 0xa8, 0x5c, 0x70, 0xa9, 0x88, 0x4a, 0x35, 0xbc, 0xbf, 0x3d, 0xfa, 0xb0,
            0x77, 0xe9, 0x8b, 0x0f, 0x68, 0x13, 0x5f, 0x54,
        ];
        const KEYSTREAM_BLOCK_1: [u8; 64] = [
            0x81, 0xd4, 0x93, 0x3f, 0x8b, 0x32, 0x2a, 0xc0, 0xcd, 0x76, 0x2c, 0x27, 0x23, 0x5c,
            0xe2, 0xb3, 0x15, 0x34, 0xe0, 0x24, 0x4a, 0x9a, 0x2f, 0x1f, 0xd5, 0xe9, 0x44, 0x98,
            0xd4, 0x7f, 0xf1, 0x08, 0x79, 0x0c, 0x00, 0x9c, 0xf9, 0xe1, 0xa3, 0x48, 0x03, 0x2a,
            0x76, 0x94, 0xcb, 0x28, 0x02, 0x4c, 0xd9, 0x6d, 0x34, 0x98, 0x36, 0x1e, 0xdb, 0x17,
            0x85, 0xaf, 0x75, 0x2d, 0x18, 0x7a, 0xb5, 0x4b,
        ];
        let (block_0, block_1) = fetch_blocks(data);
        assert!(block_0 == KEYSTREAM_BLOCK_0);
        assert!(block_1 == KEYSTREAM_BLOCK_1);
    }

    // We're only able to retrieve chacha output in blocks of 4, but we
    // only bother testing the first two blocks, discarding the rest.
    fn fetch_blocks(data: [u64; BUF_LEN]) -> ([u8; 64], [u8; 64]) {
        const BYTE_LEN: usize = size_of::<[u64; BUF_LEN]>();
        let stream_as_bytes: [u8; BYTE_LEN] = unsafe { transmute(data) };
        let block_0: [u8; 64] = stream_as_bytes[..64].try_into().unwrap();
        let block_1: [u8; 64] = stream_as_bytes[64..128].try_into().unwrap();
        (block_0, block_1)
    }
}
