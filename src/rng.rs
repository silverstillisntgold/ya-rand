use core::ptr::swap;
#[cfg(feature = "alloc")]
use {
    crate::ya_rand_encoding::Encoder,
    alloc::{boxed::Box, string::String},
};

const F64_MANT: u32 = f64::MANTISSA_DIGITS;
const F32_MANT: u32 = f32::MANTISSA_DIGITS;
const F64_MAX_PRECISE: u64 = 1 << F64_MANT;
const F32_MAX_PRECISE: u64 = 1 << F32_MANT;
const F64_DIVISOR: f64 = F64_MAX_PRECISE as f64;
const F32_DIVISOR: f32 = F32_MAX_PRECISE as f32;
pub const ALPHANUMERIC: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

/// Trait for RNGs that are known to provide streams of cryptographically secure data.
pub trait SecureYARandGenerator: YARandGenerator {
    /// Fills `dst` with random data, which is safe to be used in cryptographic contexts.
    ///
    /// # Examples
    ///
    /// ```
    /// use ya_rand::*;
    ///
    /// let mut rng = new_rng_secure();
    /// let mut data = [0; 1738];
    /// rng.fill_bytes(&mut data);
    /// assert!(data.into_iter().any(|v| v != 0));
    /// ```
    fn fill_bytes(&mut self, dst: &mut [u8]);

    /// Generates a random `String` with length `len`, using the provided
    /// `Encoder` to determine character set and minimum secure length. Since
    /// character sets can only contain valid ascii values, the length of the created
    /// `String` reprensents both the size of the `String` in bytes, and the
    /// amount of characters it contains.
    ///
    /// Returns `None` when `len` is less than what the encoder declares to be secure.
    ///
    /// All provided encoders are accessible through [`crate::ya_rand_encoding`].
    /// Users wishing to implement their own encoding must do so through the
    /// [`Encoder`](crate::ya_rand_encoding::Encoder) trait.
    ///
    /// Originally inspired by golang's addition of [`rand.Text`] in release 1.24,
    /// but altered to be encoding/length generic and unbiased for non-trivial bases.
    ///
    /// [`rand.Text`]:
    /// https://cs.opensource.google/go/go/+/refs/tags/go1.24.0:src/crypto/rand/text.go
    ///
    /// # Examples
    ///
    /// ```
    /// use ya_rand::*;
    /// use ya_rand_encoding::Base16Lowercase as B16L;
    ///
    /// const LEN: usize = 420;
    /// let mut rng = new_rng_secure();
    /// // Safe to unwrap because 420 is greater than 32, which
    /// // is the minimum secure length of `Base16Lowercase`.
    /// let hex_string = rng.text::<B16L>(LEN).unwrap();
    /// assert!(hex_string.len() == LEN);
    /// assert!(hex_string.bytes().count() == LEN);
    /// assert!(hex_string.chars().count() == LEN);
    /// for c in hex_string.bytes() {
    ///     assert!(
    ///         (b'0' <= c && c <= b'9') ||
    ///         (b'a' <= c && c <= b'f')
    ///     );
    /// }
    /// ```
    #[cfg(feature = "alloc")]
    fn text<E: Encoder>(&mut self, len: usize) -> Option<String> {
        match len >= E::MIN_LEN {
            true => Some({
                const BYTE_VALUES: usize = 1 << u8::BITS;
                // SAFETY: u8's are a trivial type and we pwomise to
                // always overwrite all of them UwU.
                let mut data = unsafe { Box::new_uninit_slice(len).assume_init().into_vec() };
                // This branch is evaluated at compile time, so concrete
                // implementations in final binaries will only have the
                // contents of the branch suitable for the encoder used.
                if BYTE_VALUES % E::CHARSET.len() == 0 {
                    self.fill_bytes(&mut data);
                    // Directly map each random u8 to a character in the set.
                    // Since this branch is only reachable when length is a power
                    // of two, the modulo gets optimized out and the whole thing
                    // gets vectorized. The assembly for this branch is
                    // absolutely beautiful.
                    // This approach is extremely efficient, but only produces
                    // unbiased random sequences when the length of the current
                    // `CHARSET` is divisible by the amount of possible u8 values,
                    // which is why we need a fallback approach.
                    for d in &mut data {
                        let random_value = *d as usize;
                        *d = E::CHARSET[random_value % E::CHARSET.len()];
                    }
                } else {
                    // Alternative approach that's potentially much slower,
                    // but always produces unbiased results.
                    // The unwrap gets optimized out since rustc can see that
                    // `E::CHARSET` has a non-zero length.
                    data.fill_with(|| *self.choose(E::CHARSET).unwrap());
                }
                // SAFETY: All provided encoders only use ascii values, and
                // custom `Encoder` implementations agree to do the same when
                // implementing the trait.
                unsafe { String::from_utf8_unchecked(data) }
            }),
            false => None,
        }
    }
}

/// Trait for RNGs that can be created from a user-provided seed.
pub trait SeedableYARandGenerator: YARandGenerator + Default {
    /// Creates a generator from the output of an internal PRNG,
    /// which is itself seeded from `seed`.
    ///
    /// As a rule: unless you are **absolutely certain** that you need to manually
    /// seed a generator, you don't.
    /// Instead, use [`crate::new_rng`] when you need to create a new instance.
    ///
    /// If you have a scenario where you really do need a set seed, prefer to use the
    /// `Default` implementation of the desired generator.
    ///
    /// # Examples
    ///
    /// ```
    /// use ya_rand::*;
    ///
    /// // In actual use these would be declared `mut`.
    /// let rng1 = ShiroRng::new_with_seed(0);
    /// let rng2 = ShiroRng::default();
    /// // Default is just a shortcut for manually seeding with 0.
    /// assert!(rng1 == rng2);
    /// ```
    fn new_with_seed(seed: u64) -> Self;
}

/// Base trait that all RNGs must implement.
pub trait YARandGenerator: Sized {
    /// Creates a generator using randomness provided by the OS.
    ///
    /// Unlike [`YARandGenerator::new`], which will panic on failure, `try_new`
    /// propagates the error-handling responsibility to the user. But the probability
    /// of your operating systems RNG failing is absurdly low, and in the rare case that it
    /// does fail, that's not really an issue most users are going to be able to address.
    ///
    /// Stick to using [`crate::new_rng`], unless you really need a generator of a
    /// different type (you probably don't), then use `new` on your desired type.
    fn try_new() -> Result<Self, getrandom::Error>;

    /// Returns a uniformly distributed `u64` in the interval [0, 2<sup>64</sup>).
    fn u64(&mut self) -> u64;

    /// Creates a generator using randomness provided by the OS.
    ///
    /// It is recommended to use the top-level [`crate::new_rng`] instead
    /// of calling this function on a specific generator type.
    ///
    /// # Safety
    ///
    /// This function will panic if your OS rng fails to provide enough entropy.
    /// But this is extremely unlikely, and unless you're working at the kernel level it's
    /// not something you should ever be concerned with.
    ///
    /// Since Windows 10 this function is infallible, thanks to modern Windows versions adopting
    /// a user-space cryptographic architecture that can't fail during runtime.
    ///
    /// # Examples
    ///
    /// ```
    /// use ya_rand::*;
    ///
    /// // Recommended usage
    /// let mut rng1 = new_rng();
    /// // More explicit
    /// let mut rng2 = ShiroRng::new();
    /// // Even more explicit
    /// let mut rng3 = Xoshiro256pp::new();
    /// // Since these are all created using OS entropy, the odds of
    /// // their initial states colliding is vanishingly small.
    /// assert!(rng1 != rng2);
    /// assert!(rng1 != rng3);
    /// assert!(rng2 != rng3);
    /// ```
    #[inline]
    fn new() -> Self {
        Self::try_new().expect(
            "WARNING: retrieving random data from the operating system should never fail; \
            something has gone terribly wrong",
        )
    }

    /// Returns a uniformly distributed `u32` in the interval [0, 2<sup>32</sup>).
    #[inline]
    fn u32(&mut self) -> u32 {
        self.bits(u32::BITS) as u32
    }

    /// Returns a uniformly distributed `u16` in the interval [0, 2<sup>16</sup>).
    #[inline]
    fn u16(&mut self) -> u16 {
        self.bits(u16::BITS) as u16
    }

    /// Returns a uniformly distributed `u8` in the interval [0, 2<sup>8</sup>).
    #[inline]
    fn u8(&mut self) -> u8 {
        self.bits(u8::BITS) as u8
    }

    /// Returns a uniformly distributed `u64` in the interval [0, 2<sup>`bit_count`</sup>).
    ///
    /// The value of `bit_count` is clamped to 64.
    #[inline]
    fn bits(&mut self, bit_count: u32) -> u64 {
        self.u64() >> (u64::BITS - bit_count.min(u64::BITS))
    }

    /// A simple coinflip, returning a `bool` that has a 50% chance of being true.
    ///
    /// # Examples
    ///
    /// ```
    /// use ya_rand::*;
    ///
    /// const ITERATIONS: u64 = 1 << 24;
    /// let mut rng = new_rng();
    /// let mut yes: u64 = 0;
    /// let mut no: u64 = 0;
    /// for _ in 0..ITERATIONS {
    ///     if rng.bool() {
    ///         yes += 1;
    ///     } else {
    ///         no += 1;
    ///     }
    /// }
    /// // We expect the difference to be within ~5%.
    /// let THRESHOLD: u64 = ITERATIONS / 20;
    /// assert!(yes.abs_diff(no) <= THRESHOLD);
    /// ```
    #[inline]
    fn bool(&mut self) -> bool {
        // Compiles to a single "shr 63" instruction.
        self.bits(1) == 1
    }

    /// Returns a uniformly distributed `u64` in the interval [0, `max`).
    ///
    /// Using [`YARandGenerator::bits`] when `max` happens to be a power of 2
    /// will be significantly faster.
    ///
    /// # Examples
    ///
    /// ```
    /// use ya_rand::*;
    ///
    /// let mut rng = new_rng();
    /// // Special case: bound of 0 always returns 0.
    /// assert!(rng.bound(0) == 0);
    /// for i in 1..=2000 {
    ///     for _ in 0..(i * 2) {
    ///         assert!(rng.bound(i) < i);
    ///     }
    /// }
    /// ```
    #[inline]
    fn bound(&mut self, max: u64) -> u64 {
        // Lemire's method: https://arxiv.org/abs/1805.10941.
        let mut mul = || crate::util::wide_mul(self.u64(), max);
        let (mut high, mut low) = mul();
        match low < max {
            false => {}
            true => {
                let threshold = max.wrapping_neg() % max;
                while low < threshold {
                    (high, low) = mul();
                }
            }
        }
        debug_assert!(
            (max != 0 && high < max) || high == 0,
            "BUG: this assertion should be unreachable"
        );
        high
    }

    /// Returns a uniformly distributed `u64` in the interval \[0, `max`\].
    ///
    /// It is expected that `max` < `u64::MAX`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ya_rand::*;
    ///
    /// let mut rng = new_rng();
    /// for i in 0..=2000 {
    ///     for _ in 0..(i * 2) {
    ///         assert!(rng.bound_inclusive(i) <= i);
    ///     }
    /// }
    /// ```
    #[inline]
    fn bound_inclusive(&mut self, max: u64) -> u64 {
        self.bound(max + 1)
    }

    /// Returns a uniformly distributed `i64` in the interval [`min`, `max`)
    ///
    /// It is expected that `min` < `max`.
    #[inline]
    fn range(&mut self, min: i64, max: i64) -> i64 {
        let delta = max.abs_diff(min);
        (self.bound(delta) as i64) + min
    }

    /// Returns a uniformly distributed `i64` in the interval \[`min`, `max`\]
    ///
    /// It is expected that `min` <= `max` and `max` < `u64::MAX`.
    #[inline]
    fn range_inclusive(&mut self, min: i64, max: i64) -> i64 {
        self.range(min, max + 1)
    }

    /// Returns a uniformly distributed `f64` in the interval [0.0, 1.0).
    #[inline]
    fn f64(&mut self) -> f64 {
        self.bits(F64_MANT) as f64 / F64_DIVISOR
    }

    /// Returns a uniformly distributed `f32` in the interval [0.0, 1.0).
    #[inline]
    fn f32(&mut self) -> f32 {
        self.bits(F32_MANT) as f32 / F32_DIVISOR
    }

    /// Returns a uniformly distributed `f64` in the interval (0.0, 1.0].
    #[inline]
    fn f64_nonzero(&mut self) -> f64 {
        // Interval of (0, 2^53]
        let nonzero = self.bits(F64_MANT) + 1;
        nonzero as f64 / F64_DIVISOR
    }

    /// Returns a uniformly distributed `f32` in the interval (0.0, 1.0].
    #[inline]
    fn f32_nonzero(&mut self) -> f32 {
        // Interval of (0, 2^24]
        let nonzero = self.bits(F32_MANT) + 1;
        nonzero as f32 / F32_DIVISOR
    }

    /// Returns a uniformly distributed `f64` in the interval (-1.0, 1.0).
    #[inline]
    fn f64_wide(&mut self) -> f64 {
        // This approach is faster than using Generator::range.
        const BITS: u32 = F64_MANT + 1;
        const OFFSET: i64 = F64_MAX_PRECISE as i64;
        let mut x: i64;
        loop {
            // Start with an interval of [0, 2^54)
            x = self.bits(BITS) as i64;
            // Interval is now (0, 2^54)
            match x != 0 {
                true => break,
                false => {}
            }
        }
        // Shift interval to (-2^53, 2^53)
        x -= OFFSET;
        x as f64 / F64_DIVISOR
    }

    /// Returns a uniformly distributed `f32` in the interval (-1.0, 1.0).
    #[inline]
    fn f32_wide(&mut self) -> f32 {
        // This approach is faster than using Generator::range.
        const BITS: u32 = F32_MANT + 1;
        const OFFSET: i64 = F32_MAX_PRECISE as i64;
        let mut x: i64;
        loop {
            // Start with an interval of [0, 2^25)
            x = self.bits(BITS) as i64;
            // Interval is now (0, 2^25)
            match x != 0 {
                true => break,
                false => {}
            }
        }
        // Shift interval to (-2^24, 2^24)
        x -= OFFSET;
        x as f32 / F32_DIVISOR
    }

    /// Returns two indepedent and normally distributed `f64` values with
    /// a `mean` of `0.0` and a `stddev` of `1.0`.
    #[cfg(feature = "std")]
    fn f64_normal(&mut self) -> (f64, f64) {
        // Marsaglia polar method.
        // TLDR: It projects a point **within** the unit
        // circle onto the unit radius.
        let mut x: f64;
        let mut y: f64;
        let mut s: f64;
        loop {
            x = self.f64_wide();
            y = self.f64_wide();
            s = (x * x) + (y * y);
            // Reroll if s does not lie **within** the unit circle.
            match s < 1.0 && s != 0.0 {
                true => break,
                false => {}
            }
        }
        let t = (2.0 * s.ln().abs() / s).sqrt();
        (x * t, y * t)
    }

    /// Returns two indepedent and normally distributed `f64` values with
    /// user-defined `mean` and `stddev`.
    ///
    /// It is expected that `stddev.abs()` != `0.0`.
    #[cfg(feature = "std")]
    #[inline]
    fn f64_normal_distribution(&mut self, mean: f64, stddev: f64) -> (f64, f64) {
        let (x, y) = self.f64_normal();
        ((x * stddev) + mean, (y * stddev) + mean)
    }

    /// Returns an exponentially distributed `f64` with `lambda` of `1.0`.
    #[cfg(feature = "std")]
    #[inline]
    fn f64_exponential(&mut self) -> f64 {
        // Using abs() instead of negating the result of ln()
        // to avoid outputs of -0.0.
        self.f64_nonzero().ln().abs()
    }

    /// Returns an exponentially distributed `f64` with user-defined `lambda`.
    ///
    /// It is expected that `lambda.abs()` != `0.0`.
    #[cfg(feature = "std")]
    #[inline]
    fn f64_exponential_lambda(&mut self, lambda: f64) -> f64 {
        self.f64_exponential() / lambda
    }

    /// Returns a randomly chosen item from the iterator of `collection`.
    ///
    /// Returns `None` when the length of the iterator is zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use ya_rand::*;
    ///
    /// const SIZE: usize = 1738;
    /// let mut rng = new_rng();
    /// let mut v = [0; SIZE];
    /// for i in 0..SIZE {
    ///     v[i] = i;
    /// }
    /// let (top, bottom) = v.split_at(v.len() / 2);
    ///
    /// // Sanity check.
    /// let random_item = rng.choose(&v).expect("vector `v` is not empty");
    /// assert!(v.contains(random_item));
    ///
    /// // Choose `random_item` from the top half of the array.
    /// let random_item = rng.choose(top).expect("still not empty");
    /// assert!(top.contains(random_item) == true);
    ///
    /// // We're looking in the bottom half so we won't find the
    /// // `random_item` from the top half.
    /// assert!(bottom.contains(random_item) == false);
    /// ```
    #[inline]
    fn choose<C>(&mut self, collection: C) -> Option<C::Item>
    where
        C: IntoIterator,
        C::IntoIter: ExactSizeIterator,
    {
        let mut iter = collection.into_iter();
        let len = iter.len();
        match len != 0 {
            true => Some({
                let idx = self.bound(len as u64) as usize;
                // SAFETY: Since `bound` always returns a value less than
                // it's input, `nth` will never return `None`.
                unsafe { iter.nth(idx).unwrap_unchecked() }
            }),
            false => None,
        }
    }

    /// Returns a randomly selected ASCII character from the pool of:
    ///
    /// `'A'..='Z'`, and`'a'..='z'`
    #[inline]
    fn ascii_alphabetic(&mut self) -> char {
        *self.choose(&ALPHANUMERIC[..52]).unwrap() as char
    }

    /// Returns a randomly selected ASCII character from the pool of:
    ///
    /// `'A'..='Z'`
    #[inline]
    fn ascii_uppercase(&mut self) -> char {
        *self.choose(&ALPHANUMERIC[..26]).unwrap() as char
    }

    /// Returns a randomly selected ASCII character from the pool of:
    ///
    /// `'a'..='z'`
    #[inline]
    fn ascii_lowercase(&mut self) -> char {
        *self.choose(&ALPHANUMERIC[26..52]).unwrap() as char
    }

    /// Returns a randomly selected ASCII character from the pool of:
    ///
    /// `'A'..='Z'`, `'a'..='z'`, and `'0'..='9'`
    #[inline]
    fn ascii_alphanumeric(&mut self) -> char {
        *self.choose(&ALPHANUMERIC[..]).unwrap() as char
    }

    /// Returns a randomly selected ASCII character from the pool of:
    ///
    /// `'0'..='9'`
    #[inline]
    fn ascii_digit(&mut self) -> char {
        *self.choose(&ALPHANUMERIC[52..]).unwrap() as char
    }

    /// Performs a Fisher-Yates shuffle on the contents of `slice`.
    ///
    /// This implementation is the modern variant introduced by
    /// Richard Durstenfeld. It is in-place and O(n).
    ///
    /// # Examples
    ///
    /// ```
    /// use ya_rand::*;
    ///
    /// let mut rng = new_rng();
    /// let mut data = [0; 1738];
    /// for i in 0..data.len() {
    ///     data[i] = i;
    /// }
    /// assert!(data.is_sorted() == true);
    ///
    /// rng.shuffle(&mut data);
    /// assert!(data.is_sorted() == false);
    /// ```
    fn shuffle<T>(&mut self, slice: &mut [T]) {
        let slice_ptr = slice.as_mut_ptr();
        for i in (1..slice.len()).rev() {
            let j = self.bound_inclusive(i as u64) as usize;
            // SAFETY: Index 'i' will always be in bounds because it's
            // bounded by slice length; index 'j' will always be
            // in bounds because it's bounded by 'i'.
            unsafe {
                swap(slice_ptr.add(i), slice_ptr.add(j));
            }
        }
    }
}
