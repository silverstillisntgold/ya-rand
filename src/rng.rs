const F64_MANT: u32 = f64::MANTISSA_DIGITS;
const F32_MANT: u32 = f32::MANTISSA_DIGITS;
const F64_MAX_PRECISE: u64 = 1 << F64_MANT;
const F32_MAX_PRECISE: u64 = 1 << F32_MANT;
const F64_DIVISOR: f64 = F64_MAX_PRECISE as f64;
const F32_DIVISOR: f32 = F32_MAX_PRECISE as f32;
const ASCII_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

#[cfg(feature = "secure")]
pub trait SecureGenerator {
    /// Fills `dest` with random data, which is safe
    /// to be used in cryptographic contexts.
    ///
    /// # Examples
    /// ```
    /// use ya_rand::*;
    ///
    /// let mut rng = new_rng_secure();
    /// let mut data = [0; 1738];
    /// rng.fill_bytes(&mut data);
    /// assert!(data.iter().any(|v| *v != 0));
    /// ```
    fn fill_bytes(&mut self, dest: &mut [u8]);
}

pub trait SeedableGenerator {
    /// Creates a generator from the output of an internal SplitMix64 generator,
    /// which is itself seeded using `seed`.
    ///
    /// As a rule: unless you are **absolutely certain** that you need to manually
    /// seed a generator, you don't.
    /// Instead, use [`crate::new_rng`] when you need to create a new instance.
    ///
    /// If you have a scenario where you really need a set seed, prefer to use the `Default`
    /// implementation of the desired generator.
    ///
    /// # Examples
    /// ```
    /// use ya_rand::*;
    ///
    /// let mut rng1 = ShiroRng::new_with_seed(0);
    /// let mut rng2 = ShiroRng::default();
    /// // Default is just a shortcut for manually seeding with 0.
    /// assert!(rng1 == rng2);
    /// ```
    fn new_with_seed(seed: u64) -> Self;
}

pub trait Generator: Sized {
    /// Creates a generator using randomness provided by the OS.
    ///
    /// Unlike [`Generator::new`], which will panic on failure, `try_new`
    /// propagates the error-handling responsibility to the user. That being
    /// said, the probability of your operating systems RNG failing is absurdly
    /// low. And in the case that is does fail, that's not really an issue
    /// most users are going to be able to address.
    ///
    /// Stick to using [`crate::new_rng`], unless you **need** a generator of a
    /// different type (and you probably don't), then use `new` on your desired type.
    fn try_new() -> Result<Self, getrandom::Error>;

    /// Creates a generator using randomness provided by the OS.
    ///
    /// It is recommended to instead use the top-level [`crate::new_rng`] instead
    /// of calling this function on a specific generator type.
    ///
    /// # Examples
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
    /// // their states colliding should be vanishingly small.
    /// assert!(rng1 != rng2);
    /// assert!(rng1 != rng3);
    /// assert!(rng2 != rng3);
    /// ```
    fn new() -> Self {
        Self::try_new().expect(
            "WARNING: retrieving random data from the operating system should never fail; \
            something has gone terribly wrong",
        )
    }

    /// Returns a uniformly distributed u64 in the interval [0, 2<sup>64</sup>).
    fn u64(&mut self) -> u64;

    /// Returns a uniformly distributed u32 in the interval [0, 2<sup>32</sup>).
    #[inline]
    fn u32(&mut self) -> u32 {
        self.bits(u32::BITS) as u32
    }

    /// Returns a uniformly distributed u16 in the interval [0, 2<sup>16</sup>).
    #[inline]
    fn u16(&mut self) -> u16 {
        self.bits(u16::BITS) as u16
    }

    /// Returns a uniformly distributed u8 in the interval [0, 2<sup>8</sup>).
    #[inline]
    fn u8(&mut self) -> u8 {
        self.bits(u8::BITS) as u8
    }

    /// Returns a uniformly distributed u64 in the interval [0, 2<sup>`bit_count`</sup>).
    #[inline]
    fn bits(&mut self, bit_count: u32) -> u64 {
        debug_assert!(bit_count <= u64::BITS);
        self.u64() >> (u64::BITS - bit_count)
    }

    /// Returns a bool with 50% odds of being true.
    ///
    /// A simple coinflip.
    ///
    /// # Examples
    /// ```
    /// use ya_rand::*;
    ///
    /// const ITER_COUNT: u64 = 1 << 24;
    /// let mut rng = new_rng();
    /// let mut ones: u64 = 0;
    /// let mut zeroes: u64 = 0;
    /// for _ in 0..ITER_COUNT {
    ///     if rng.bool() {
    ///         ones += 1;
    ///     } else {
    ///         zeroes += 1;
    ///     }
    /// }
    /// // We expect the difference to be within ~5%.
    /// let THRESHOLD: u64 = ITER_COUNT / 20;
    /// assert!(ones.abs_diff(zeroes) <= THRESHOLD);
    /// ```
    #[inline]
    fn bool(&mut self) -> bool {
        self.bits(1) == 1
    }

    /// Returns a uniformly distributed u64 in the interval [0, `bound`).
    ///
    /// Using [`Generator::bits`] when `bound` happens to be a power of 2
    /// is faster and generates less machine code.
    ///
    /// # Examples
    /// ```
    /// use ya_rand::*;
    ///
    /// let mut rng = new_rng();
    /// for i in 1..=4000 {
    ///     let iters = 64.max(i * 2);
    ///     for _ in 0..iters {
    ///         assert!(rng.bound(i) < i);
    ///     }
    /// }
    /// ```
    #[inline]
    fn bound(&mut self, bound: u64) -> u64 {
        use crate::util::wide_mul;
        let (mut high, mut low) = wide_mul(self.u64(), bound);
        // Will nearly always be false when `bound` isn't close to u64::MAX.
        match low < bound {
            false => {}
            true => {
                // Can actually be a pretty cheap failure branch, since
                // rustc can compute `threshold` at compile time when `bound`
                // is a constant.
                let threshold = bound.wrapping_neg() % bound;
                while low < threshold {
                    (high, low) = wide_mul(self.u64(), bound);
                }
            }
        }
        debug_assert!((bound != 0 && high < bound) || (high == 0));
        high
    }

    /// Returns a uniformly distributed u64 in the interval \[0, `bound`\].
    ///
    /// # Examples
    /// ```
    /// use ya_rand::*;
    ///
    /// let mut rng = new_rng();
    /// for i in 0..=4000 {
    ///     let iters = 64.max(i * 2);
    ///     for _ in 0..iters {
    ///         assert!(rng.bound_inclusive(i) <= i);
    ///     }
    /// }
    /// ```
    #[inline]
    fn bound_inclusive(&mut self, bound: u64) -> u64 {
        self.bound(bound + 1)
    }

    /// Returns a uniformly distributed i64 in the interval [`start`, `end`)
    #[inline]
    fn range(&mut self, start: i64, end: i64) -> i64 {
        let delta = end - start;
        debug_assert!(delta > 0);
        (self.bound(delta as u64) as i64) + start
    }

    /// Returns a uniformly distributed i64 in the interval \[`start`, `end`\]
    #[inline]
    fn range_inclusive(&mut self, start: i64, end: i64) -> i64 {
        self.range(start, end + 1)
    }

    /// Returns a uniformly distributed f64 in the interval [0.0, 1.0).
    #[inline]
    fn f64(&mut self) -> f64 {
        (self.bits(F64_MANT) as f64) / F64_DIVISOR
    }

    /// Returns a uniformly distributed f32 in the interval [0.0, 1.0).
    #[inline]
    fn f32(&mut self) -> f32 {
        (self.bits(F32_MANT) as f32) / F32_DIVISOR
    }

    /// Returns a uniformly distributed f64 in the interval (0.0, 1.0].
    #[inline]
    fn f64_nonzero(&mut self) -> f64 {
        // Interval of (0, 2^53]
        let nonzero = self.bits(F64_MANT) + 1;
        (nonzero as f64) / F64_DIVISOR
    }

    /// Returns a uniformly distributed f32 in the interval (0.0, 1.0].
    #[inline]
    fn f32_nonzero(&mut self) -> f32 {
        // Interval of (0, 2^24]
        let nonzero = self.bits(F32_MANT) + 1;
        (nonzero as f32) / F32_DIVISOR
    }

    /// Returns a uniformly distributed f64 in the interval (-1.0, 1.0).
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
        (x as f64) / F64_DIVISOR
    }

    /// Returns a uniformly distributed f32 in the interval (-1.0, 1.0).
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
        (x as f32) / F32_DIVISOR
    }

    /// Returns two indepedent and normally distributed f64 values with
    /// mean = 0 and stddev = 1.
    #[cfg(feature = "std")]
    #[inline]
    fn f64_normal(&mut self) -> (f64, f64) {
        // Marsaglia polar method.
        let mut x: f64;
        let mut y: f64;
        let mut s: f64;
        loop {
            x = self.f64_wide();
            y = self.f64_wide();
            s = (x * x) + (y * y);
            // Reroll if s does not lie within the unit circle
            match s < 1.0 && s != 0.0 {
                true => break,
                false => {}
            }
        }
        let t = (2.0 * s.ln().abs() / s).sqrt();
        (x * t, y * t)
    }

    /// Returns two indepedent and normally distributed f64 values with
    /// user-defined `mean` and `stddev`.
    #[cfg(feature = "std")]
    #[inline]
    fn f64_normal_distribution(&mut self, mean: f64, stddev: f64) -> (f64, f64) {
        debug_assert!(stddev != 0.0);
        let (x, y) = self.f64_normal();
        ((x * stddev) + mean, (y * stddev) + mean)
    }

    /// Returns an exponentially distributed f64 with lambda = 1.
    #[cfg(feature = "std")]
    #[inline]
    fn f64_exponential(&mut self) -> f64 {
        // Using abs() instead of negating the result of ln()
        // to avoid outputs of -0.0.
        self.f64_nonzero().ln().abs()
    }

    /// Returns an exponentially distributed f64 with user-defined `lambda`.
    #[cfg(feature = "std")]
    #[inline]
    fn f64_exponential_lambda(&mut self, lambda: f64) -> f64 {
        debug_assert!(lambda != 0.0);
        self.f64_exponential() / lambda
    }

    /// Returns a randomly selected item from the iterator of `collection`.
    ///
    /// This method will only return `None` when the length of
    /// `collection` is zero.
    ///
    /// # Examples
    /// ```
    /// use ya_rand::*;
    ///
    /// const SIZE: usize = 1738;
    /// const HALF: usize = SIZE / 2;
    /// let mut rng = new_rng();
    /// let mut v = [0; SIZE];
    /// for i in 0..SIZE {
    ///     v[i] = i;
    /// }
    ///
    /// let random_choice = rng.choice(&v).expect("Vector 'v' is not empty.");
    /// assert!(v.contains(random_choice));
    ///
    /// let random_choice = rng.choice(&v[HALF..]).expect("Still not empty.");
    /// assert!(v[HALF..].contains(random_choice) == true);
    ///
    /// // We randomly selected from the top half so we won't find
    /// // our value in the bottom half.
    /// assert!(v[..HALF].contains(random_choice) == false);
    /// ```
    #[inline]
    fn choice<C>(&mut self, collection: C) -> Option<C::Item>
    where
        C: IntoIterator,
        C::IntoIter: ExactSizeIterator,
    {
        let mut iter = collection.into_iter();
        let len = iter.len();
        if len == 0 {
            return None;
        }
        let idx = self.bound(len as u64) as usize;
        Some(unsafe { iter.nth(idx).unwrap_unchecked() })
    }

    /// Returns a randomly selected ASCII alphabetic character.
    #[inline]
    fn ascii_alphabetic(&mut self) -> char {
        unsafe { *self.choice(&ASCII_CHARS[..52]).unwrap_unchecked() as char }
    }

    /// Returns a randomly selected ASCII uppercase character.
    #[inline]
    fn ascii_uppercase(&mut self) -> char {
        unsafe { *self.choice(&ASCII_CHARS[..26]).unwrap_unchecked() as char }
    }

    /// Returns a randomly selected ASCII lowercase character.
    #[inline]
    fn ascii_lowercase(&mut self) -> char {
        unsafe { *self.choice(&ASCII_CHARS[26..52]).unwrap_unchecked() as char }
    }

    /// Returns a randomly selected ASCII alphanumeric character.
    #[inline]
    fn ascii_alphanumeric(&mut self) -> char {
        unsafe { *self.choice(&ASCII_CHARS[..]).unwrap_unchecked() as char }
    }

    /// Returns a randomly selected ASCII digit character.
    #[inline]
    fn ascii_digit(&mut self) -> char {
        unsafe { *self.choice(&ASCII_CHARS[52..]).unwrap_unchecked() as char }
    }

    /// Performs a Fisher-Yates shuffle on the contents of `slice`.
    ///
    /// This implementation is the modern variant introduced by Richard Durstenfeld.
    /// It is in-place and O(n). A slice pointer is used to avoid any bounds checks.
    ///
    /// # Examples
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
    #[inline(never)]
    fn shuffle<T>(&mut self, slice: &mut [T]) {
        let slice_ptr = slice.as_mut_ptr();
        for i in (1..slice.len()).rev() {
            let j = self.bound_inclusive(i as u64) as usize;
            // SAFETY: Index 'i' will always be in bounds because it's
            // bounded by slice length; index 'j' will always be
            // in bounds because it's bounded by 'i'.
            unsafe {
                core::ptr::swap(slice_ptr.add(i), slice_ptr.add(j));
            }
        }
    }
}
