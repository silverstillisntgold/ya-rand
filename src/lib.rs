/*!
This crate provides simple and fast pseudo random number generation.

# Usage

Glob import the contents of the library and use [`new_rng`] to create new RNGs wherever
you need them. Then call whatever method you require on that instance.

If you need cryptographic security, enable the **secure** library feature and use [`new_rng_secure`].

```
use ya_rand::*;

let mut rng = new_rng();
let max: u64 = 69;
// Yep, that's all there is to it.
let val: u64 = rng.bound(max);
assert!(val < max);
```

# Features
* **std** -
    Enabled by default. Enables normal and exponential distributions, error type conversions
    for getrandom, and SIMD optimizations in the rand_chacha crate.
* **inline** -
    Marks each [`Generator::u64`] implementation with #\[inline\]. Should generally increase
    runtime performance at the cost of binary size and maybe compile time. You'll have
    to test your specific use-case to determine how much this feature will impact you.
* **secure** -
    Enables infrastructure for cryptographically secure random number generation via the
    [`rand_chacha`] crate. Noticeably increases compile time and binary size.
*/

#![no_std]

mod rng;
mod util;
mod xoroshiro128pp;
mod xoshiro256pp;
mod xoshiro512pp;

pub use crate::rng::{Generator, SeedableGenerator};
pub use xoroshiro128pp::Xoroshiro128pp;
pub use xoshiro256pp::Xoshiro256pp;
pub use xoshiro512pp::Xoshiro512pp;

/// The recommended generator for all non-cryptographic purposes.
pub type ShiroRng = Xoshiro256pp;

/// The recommended way to create new PRNG instances.
///
/// Identical to calling [`ShiroRng::new`] or [`Xoshiro256pp::new`].
#[inline]
pub fn new_rng() -> ShiroRng {
    ShiroRng::new()
}

#[cfg(feature = "secure")]
mod secure;
#[cfg(feature = "secure")]
pub use rng::SecureGenerator;
#[cfg(feature = "secure")]
pub use secure::SecureRng;

/// The recommended way to create new CRNG instances.
///
/// Identical to calling [`SecureRng::new`].
#[cfg(feature = "secure")]
#[inline]
pub fn new_rng_secure() -> SecureRng {
    SecureRng::new()
}

#[cfg(test)]
mod test {
    extern crate std;

    use super::*;
    use std::collections::HashSet;

    const ITERATIONS: u64 = 1 << 12;
    const CAP: usize = 100;

    #[test]
    pub fn ascii_alphabetic() {
        let mut rng = new_rng();
        let mut vals = HashSet::with_capacity(CAP);
        for _ in 0..ITERATIONS {
            let result = rng.ascii_alphabetic();
            assert!(result.is_ascii_alphabetic());
            vals.insert(result);
        }
        assert!(vals.len() == 52);
    }

    #[test]
    pub fn ascii_uppercase() {
        let mut rng = new_rng();
        let mut vals = HashSet::with_capacity(CAP);
        for _ in 0..ITERATIONS {
            let result = rng.ascii_uppercase();
            assert!(result.is_ascii_uppercase());
            vals.insert(result);
        }
        assert!(vals.len() == 26);
    }

    #[test]
    pub fn ascii_lowercase() {
        let mut rng = new_rng();
        let mut vals = HashSet::with_capacity(CAP);
        for _ in 0..ITERATIONS {
            let result = rng.ascii_lowercase();
            assert!(result.is_ascii_lowercase());
            vals.insert(result);
        }
        assert!(vals.len() == 26);
    }

    #[test]
    pub fn ascii_alphanumeric() {
        let mut rng = new_rng();
        let mut vals = HashSet::with_capacity(CAP);
        for _ in 0..ITERATIONS {
            let result = rng.ascii_alphanumeric();
            assert!(result.is_ascii_alphanumeric());
            vals.insert(result);
        }
        assert!(vals.len() == 62);
    }

    #[test]
    pub fn ascii_digit() {
        let mut rng = new_rng();
        let mut vals = HashSet::with_capacity(CAP);
        for _ in 0..ITERATIONS {
            let result = rng.ascii_digit();
            assert!(result.is_ascii_digit());
            vals.insert(result);
        }
        assert!(vals.len() == 10);
    }

    #[test]
    fn wide_mul() {
        const SHIFT: u32 = 48;
        const EXPECTED_HIGH: u64 = 1 << ((SHIFT * 2) - u64::BITS);
        const EXPECTED_LOW: u64 = 0;
        let x = 1 << SHIFT;
        let y = x;
        // 2^48 * 2^48 = 2^96
        let (high, low) = crate::util::wide_mul(x, y);
        assert!(high == EXPECTED_HIGH);
        assert!(low == EXPECTED_LOW);
    }
}
