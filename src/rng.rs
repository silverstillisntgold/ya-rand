const F64_MANT: u32 = f64::MANTISSA_DIGITS;
const F32_MANT: u32 = f32::MANTISSA_DIGITS;
const F64_MAX_PRECISE: u64 = 1 << F64_MANT;
const F32_MAX_PRECISE: u64 = 1 << F32_MANT;
const F64_DIVISOR: f64 = F64_MAX_PRECISE as f64;
const F32_DIVISOR: f32 = F32_MAX_PRECISE as f32;
const ASCII_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

/// Trait for RNGs that are known to provide streams of cryptographically secure data.
#[cfg(feature = "secure")]
pub trait SecureYARandGenerator: YARandGenerator {
    /// Fills `dest` with random data, which is safe to be used
    /// in cryptographic contexts.
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
    fn fill_bytes(&mut self, dest: &mut [u8]);
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
    ///
    /// The value of `bit_count` is clamped to 64.
    #[inline]
    fn bits(&mut self, bit_count: u32) -> u64 {
        self.u64() >> (u64::BITS - bit_count.min(u64::BITS))
    }

    /// A simple coinflip, returning a bool that has a 50% chance of being true.
    ///
    /// # Examples
    ///
    /// ```
    /// use ya_rand::*;
    ///
    /// const ITER_COUNT: u64 = 1 << 24;
    /// let mut rng = new_rng();
    /// let mut yes: u64 = 0;
    /// let mut no: u64 = 0;
    /// for _ in 0..ITER_COUNT {
    ///     if rng.bool() {
    ///         yes += 1;
    ///     } else {
    ///         no += 1;
    ///     }
    /// }
    /// // We expect the difference to be within ~5%.
    /// let THRESHOLD: u64 = ITER_COUNT / 20;
    /// assert!(yes.abs_diff(no) <= THRESHOLD);
    /// ```
    #[inline]
    fn bool(&mut self) -> bool {
        // Compiles to a single "shr 63" instruction.
        self.bits(1) == 1
    }

    /// Returns a uniformly distributed u64 in the interval [0, `max`).
    ///
    /// Using [`YARandGenerator::bits`] when `max` happens to be a power of 2
    /// is faster and generates better assembly.
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
        use crate::util::wide_mul;
        let (mut high, mut low) = wide_mul(self.u64(), max);
        match low < max {
            false => {}
            true => {
                let threshold = max.wrapping_neg() % max;
                while low < threshold {
                    (high, low) = wide_mul(self.u64(), max);
                }
            }
        }
        debug_assert!((max != 0 && high < max) || high == 0);
        high
    }

    /// Returns a uniformly distributed u64 in the interval \[0, `max`\].
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

    /// Returns a uniformly distributed i64 in the interval [`min`, `max`)
    #[inline]
    fn range(&mut self, min: i64, max: i64) -> i64 {
        let delta = max - min;
        debug_assert!(delta > 0);
        (self.bound(delta as u64) as i64) + min
    }

    /// Returns a uniformly distributed i64 in the interval \[`min`, `max`\]
    #[inline]
    fn range_inclusive(&mut self, min: i64, max: i64) -> i64 {
        self.range(min, max + 1)
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
    /// `mean` = 0.0 and `stddev` = 1.0.
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
            // Reroll if s does not lie **within** the unit circle.
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

    /// Returns an exponentially distributed f64 with `lambda` = 1.0.
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

    /// Returns a randomly chosen item from the iterator of `collection`.
    ///
    /// This method will only return `None` when the length of
    /// `collection` is zero.
    ///
    /// # Examples
    ///
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
    /// let random_choice = rng.choose(&v).expect("Vector 'v' is not empty.");
    /// assert!(v.contains(random_choice));
    ///
    /// let random_choice = rng.choose(&v[HALF..]).expect("Still not empty.");
    /// assert!(v[HALF..].contains(random_choice) == true);
    ///
    /// // We randomly selected from the top half so we won't find
    /// // our value in the bottom half.
    /// assert!(v[..HALF].contains(random_choice) == false);
    /// ```
    #[inline]
    fn choose<C>(&mut self, collection: C) -> Option<C::Item>
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
        unsafe { *self.choose(&ASCII_CHARS[..52]).unwrap_unchecked() as char }
    }

    /// Returns a randomly selected ASCII uppercase character.
    #[inline]
    fn ascii_uppercase(&mut self) -> char {
        unsafe { *self.choose(&ASCII_CHARS[..26]).unwrap_unchecked() as char }
    }

    /// Returns a randomly selected ASCII lowercase character.
    #[inline]
    fn ascii_lowercase(&mut self) -> char {
        unsafe { *self.choose(&ASCII_CHARS[26..52]).unwrap_unchecked() as char }
    }

    /// Returns a randomly selected ASCII alphanumeric character.
    #[inline]
    fn ascii_alphanumeric(&mut self) -> char {
        unsafe { *self.choose(&ASCII_CHARS[..]).unwrap_unchecked() as char }
    }

    /// Returns a randomly selected ASCII digit character.
    #[inline]
    fn ascii_digit(&mut self) -> char {
        unsafe { *self.choose(&ASCII_CHARS[52..]).unwrap_unchecked() as char }
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
