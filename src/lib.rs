/*!
This crate provides simple and fast pseudo random number generation.

# Usage

TODO

# Features
* **std** -
    Enabled by default.
* **inline** -
    Might improve speed.
* **secure** -
    Enables Cryptographic RNG infrastructure.
*/

#![no_std]

#[cfg(feature = "std")]
extern crate std;

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

/// The recommended way to create new Pseudo-RNG instances.
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

/// The recommended way to create new Crypto-RNG instances.
///
/// Identical to calling [`SecureRng::new`].
#[cfg(feature = "secure")]
#[inline]
pub fn new_rng_secure() -> SecureRng {
    SecureRng::new()
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashSet;

    const ITERATIONS: u64 = 1 << 13;
    const CAP: usize = 100;

    #[test]
    pub fn ascii_alphabetic() {
        let mut rng = ShiroRng::new();
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
        let mut rng = ShiroRng::new();
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
        let mut rng = ShiroRng::new();
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
        let mut rng = ShiroRng::new();
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
        let mut rng = ShiroRng::new();
        let mut vals = HashSet::with_capacity(CAP);
        for _ in 0..ITERATIONS {
            let result = rng.ascii_digit();
            assert!(result.is_ascii_digit());
            vals.insert(result);
        }
        assert!(vals.len() == 10);
    }
}
