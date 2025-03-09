/*!
"Alright you disgusting rat bastard, so how does all this garbage generate cryptographic random values?"
Take a seat little Timmy, and allow me to blow your mind.

Before anything else, it's important to have a general understanding of the structure of the
reference ChaCha algorithm. A ChaCha instance typically holds 16 32-bit integers (their signedness
is irrelevant), in the form of a 4-by-4 matrix. This being a flat or 2d array is an implementation
detail that shouldn't impact performance or output at all, and is something you don't need to
worry about.

The first 4 integers are constant values from the string "`expand 32-byte k`", and exist to ensure a
base amount of entropy for instances with shitty key values. The next 8 integers are the key/seed values.
Of the last 4 integers, the first 2 together represent a 64-bit integer that functions as the counter
for the instance. **This counter is the only thing that changes between invocations of a given ChaCha
instance.** Say you run a ChaCha round with a given state, where the 64-bit counter happens to 69. After
it has returned the result, the counter of that instance will then be 70, which will impact the next execution
of a ChaCha round. The last 2 integers (nonce values) are used as a way of differentiating between instances
that might have the same key/seed values.

```text
"expa"   "nd 3"  "2-by"  "te k"
Key      Key      Key    Key
Key      Key      Key    Key
Counter  Counter  Nonce  Nonce
```

Since we are only using ChaCha as an RNG, we randomize everything when creating instances. Meaning that we
treat the nonce values as extra key/seed values, and the counter can start anywhere in it's cycle.
This is fast, simple, and means we effectively have 2<sup>320</sup> unique data streams that can be generated.
Each of these streams can provide 1 ZiB before repeating (when the counter is incremented 2<sup>64</sup>
times back to where it started on initialization).

All implementations process four instances of chacha per invocation.

The soft implementation is the [reference implementation], but structured in a way that allows the compiler
to easily auto-vectorize the rounds. The result isn't as fast as the manually vectorized variants, but seems
to be about twice as fast the equivalent non-vectorized code. The vectorized variants were developed using
[this paper].

The process of generating data using `[SecureRng]` is as follows:

1. Take the internal `ChaCha` instance and turn it into a `Machine`. A `Machine` serves as the abstraction
layer for different architectures, and it's contents will vary depending on the flags used to compile the
final binary (this crate **does not** use runtime dispatch). But it's size will always be 256 bytes,
since it will always contain 4 distinct chacha matrixs, despite their representations being different.
This `Machine` handles incrementing the counter values of it's internal chacha blocks by 0, 1, 2, and 3.
The underlying `ChaCha` struct doesn't bother storing the constants directly, they are instead directly
loaded from static memory when creating `Machine` instances.

2. The newly created `Machine` is cloned, and the original `ChaCha` instance has it's counter incremented by
4, so next time it's called we don't get overlap in any of the internal chacha instances.

3. 4 double rounds are performed (making this a ChaCha8 implemetation). In the soft implementation this is
straightforward, but the vectorized variants take a different approach. A double round performs two rounds,
the first operates on one of the four columns, and the second operates on one of the four diagonals. To make
the vectorized approaches faster, we tranform the `Machine` state so we only ever need to perform column
rounds. Column rounds don't change much, but before each "diagonal" round, we shuffle the contents of the vectors.
This is done so we can again perform a column round, but now the column we are operating on contains the data
that just a moment ago was layed out in a diagonal. After the round is completed, this transformation is
reverted.

4. The `Machine` which has just had chacha rounds performed on it is then added to the cloned `Machine` from
step 2. The resulting `Machine` then contains the output of four independent chacha matrix computed in parallel.

5. For the soft, sse2, and neon implementations the `Machine` state is already in the layout we need it and can be
transmuted (bit-cast) directly into an array for end-user consumption. But due to how vector register indexing works,
we have to do additional work for avx2 and avx512 to make sure the layout of the results are correct. It looks a
bit convoluted but all we're doing is moving the internal 128-bit components of the 256/512 bit registers around
to make their ordering match that of the sse2 variant (which directly uses 128-bit vectors).

[reference implementation]: https://en.wikipedia.org/wiki/Salsa20#ChaCha_variant
[this paper]: https://eprint.iacr.org/2013/759

## Security

TODO
*/

#![allow(invalid_value)]

mod soft;
mod util;

use crate::{SecureYARandGenerator, YARandGenerator};
use cfg_if::cfg_if;
use core::{
    mem::{transmute, MaybeUninit},
    ptr::copy_nonoverlapping,
};
use util::*;

cfg_if! {
    if #[cfg(any(target_arch = "x86_64", target_arch = "x86"))] {
        #[cfg(feature = "nightly")]
        mod avx512;
        mod avx2;
        mod sse2;
        cfg_if! {
            if #[cfg(all(feature = "nightly", target_feature = "avx512f"))] {
                use avx512::Matrix;
            } else if #[cfg(target_feature = "avx2")] {
                use avx2::Matrix;
            } else if #[cfg(target_feature = "sse2")] {
                use sse2::Matrix;
            } else {
                use soft::Matrix;
            }
        }
    // NEON on ARM32 is both unsound and gated behind nightly.
    } else if #[cfg(all(target_feature = "neon", any(target_arch = "aarch64", target_arch = "arm64ec")))] {
        mod neon;
        use neon::Matrix;
    } else {
        use soft::Matrix;
    }
}

/// A cryptographically secure random number generator.
///
/// The current implementation is ChaCha with 8 rounds.
pub struct SecureRng {
    index: usize,
    buf: [u64; BUF_LEN],
    internal: ChaCha<Matrix>,
}

impl SecureYARandGenerator for SecureRng {
    #[inline(never)]
    fn fill_bytes(&mut self, dst: &mut [u8]) {
        unsafe {
            const LEN: usize = size_of::<[u64; BUF_LEN]>();
            dst.chunks_exact_mut(LEN).for_each(|chunk| {
                let chunk_ref: &mut [u8; LEN] = chunk.try_into().unwrap();
                let chunk_reinterpreted: &mut [u64; BUF_LEN] = transmute(chunk_ref);
                self.internal.block(chunk_reinterpreted);
            });
            let remaining_chunk = dst.chunks_exact_mut(LEN).into_remainder();
            if remaining_chunk.len() != 0 {
                let mut buf = MaybeUninit::uninit().assume_init();
                self.internal.block(&mut buf);
                copy_nonoverlapping(
                    buf.as_ptr().cast(),
                    remaining_chunk.as_mut_ptr(),
                    remaining_chunk.len(),
                );
            }
        }
    }
}

impl YARandGenerator for SecureRng {
    fn try_new() -> Result<Self, getrandom::Error> {
        // We randomize **all** bits of the matrix, even the counter.
        // If used in a cipher this approach is completely braindead,
        // but since this is exclusively for use in a CRNG it's fine.
        let mut dest = unsafe { MaybeUninit::<[u8; CHACHA_SEED_LEN]>::uninit().assume_init() };
        crate::util::fill(&mut dest)?;
        let mut result = SecureRng {
            index: 0,
            buf: unsafe { MaybeUninit::uninit().assume_init() },
            internal: dest.into(),
        };
        result.internal.block(&mut result.buf);
        Ok(result)
    }

    #[cfg_attr(feature = "inline", inline)]
    fn u64(&mut self) -> u64 {
        if self.index >= self.buf.len() {
            self.internal.block(&mut self.buf);
            self.index = 0;
        }
        let result = self.buf[self.index];
        self.index += 1;
        result
    }
}
