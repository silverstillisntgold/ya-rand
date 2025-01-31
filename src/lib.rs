/*!
This crate provides simple and fast pseudo/crypto random number generation.

## But why?

Because `rand` is very cool and powerful, but kind of an enormous fucking pain in the ass
to use, and it's far too large and involved for someone who just a needs to flip a coin once
every 7 minutes. But if you're doing some crazy black magic computational sorcery, it almost
certainly has something you can use to complete your spell.

Other crates, like `fastrand`, `tinyrand`, or `oorandom`, fall somewhere between "I'm not sure I trust the
backing RNG" to "this API is literally just `rand` but less powerful". I wanted something easy, but
also fast and statistically robust.

So here we are.

## Usage

Glob import the contents of the library and use [`new_rng`] to create new RNGs wherever
you need them. Then call whatever method you require on those instances. All methods available
are directly accessible through any generator instance via the dot operator, and are named
in a way that should make it easy to quickly identify what you need.

If you need cryptographic security, enable the **secure** library feature and use
[`new_rng_secure`] instead.

"How do I access the thread-local RNG?" There isn't one, and unless Rust finds a way to significantly
improve the performance of the TLS implementation, there probably won't ever be. Create a local instance
when and where you need one, and use it while you need it.

```
use ya_rand::*;

let mut rng = new_rng();
let max: u64 = 69;
// Yep, that's all there is to it.
let val: u64 = rng.bound(max);
assert!(val < max);
```

## Features

* **std** -
    Enabled by default, but can be disabled for compatibility with `no_std` environments.
    Enables normal and exponential distributions, error type conversions
    for getrandom, and SIMD optimizations in the [`rand_chacha`] crate.
* **inline** -
    Marks each [`Generator::u64`] implementation with #\[inline\]. Should generally increase
    runtime performance at the cost of binary size and maybe compile time. You'll have
    to test your specific use-case to determine how much this feature will impact you.
* **secure** -
    Enables infrastructure for cryptographically secure random number generation via the
    [`rand_chacha`] crate. Noticeably increases compile time and binary size.

## Details

This crate uses the [xoshiro] family of pseudo-random number generators. These generators are
very fast (sub-ns when inlined), of [very high statistical quality], and small. They aren't cryptograpically
secure, but most users don't need their RNG to be secure, they just need it to be random and fast.
This crate is intended to satisfy those needs, while also being easy to use and simple to understand. It
also happens to be small and relatively fast to compile.

[xoshiro]: https://prng.di.unimi.it/
[very high statistical quality]: https://vigna.di.unimi.it/ftp/papers/ScrambledLinear.pdf

All generators output a distinct `u64` value on each call, and the various methods used for transforming
those outputs into more usable forms are all high-quality and well-understood. Placing an upper bound
on these values uses [Lemire's method]. Doing this inclusively or within a given range are both
applications of this same method with simple intermediary steps to alter the bound and apply shifts
when needed. This approach is unbiased and quite fast, but for very large bounds performance might degrade
slightly, since the algorithm needs to sample the underlying RNG more times to get an unbiased result.
If you know your bounds ahead of time, passing them as constants can help this issue, since the initial
division can be done at compile time when the value is known. Even better, if your bound happens to be a
power of 2, always use [`Generator::bits`], since it's nothing more than a bitshift of the `u64` provided
by the RNG, and will always be as fast as possible.

Floating point values (besides the normal and exponential distributions) are all generated to be uniform,
with all their values being equidistant within their provided interval. They are **not** maximally dense,
if that's something you need you'll have to generate those values yourself. This approach is very fast, and
endorsed by both [Lemire] and [Vigna] (the author of the RNGs used in this crate). The normal distribution
is generated using the [Marsaglia polar method], so it returns a pair of independently sampled `f64` values.
Exponential variates are generated using [this approach].

[Lemire's method]: https://arxiv.org/abs/1805.10941
[Lemire]: https://lemire.me/blog/2017/02/28/how-many-floating-point-numbers-are-in-the-interval-01/
[Vigna]: https://prng.di.unimi.it/#remarks
[Marsaglia polar method]: https://en.wikipedia.org/wiki/Marsaglia_polar_method
[this approach]: https://en.wikipedia.org/wiki/Exponential_distribution#Random_variate_generation

## Security

If you're in the market for secure random number generation, you'll want to enable the **secure**
feature, which provides [`SecureRng`] and the [`SecureGenerator`] trait. It functions identically to the
other provided RNGs, but with the addition of [`SecureGenerator::fill_bytes`]. The current implementation
uses ChaCha with 8 rounds via the [`rand_chacha`] crate. Unfortunately, this crate brings in a million other
dependencies and completely balloons compile times. In the future I'd like to look into doing a custom
implementation of ChaCha, but no timeline on that. Why only 8 rounds? Because people who are very
passionate about cryptography are convinced that's enough, and I have zero reason to doubt them, nor any
capacity to prove them wrong. See the [top of page 14].

The security promises made to the user are identical to those made by ChaCha as an algorithm; it is up
to you to determine if it's security guarantees are enough for your use-case given the way you intend
to use it's generated values.

[top of page 14]: https://eprint.iacr.org/2019/1492

## Safety

In the pursuit of consistent performance and no runtime failures, there are no checks performed during
runtime in release mode. This means there are a couple areas where the end-user is able to receive garbage
after providing garbage. It is expected of the user to provide reasonable values where there is an input to
be given: values shouldn't be on the verge of overflow and ranges should always have an end larger than their
start. There is minimal `unsafe` used, only in areas which directly benefit from it, and they are all brief
and easily determined to have no ill side-effects.
*/

#![no_std]

mod rng;
mod util;
mod xoshiro256pp;
mod xoshiro512pp;

pub use crate::rng::{Generator, SeedableGenerator};
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
    #[cfg(not(feature = "std"))]
    compile_error!("tests can only be run when the `std` feature is enabled");

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
        let (high, low) = util::wide_mul(x, y);
        assert!(high == EXPECTED_HIGH);
        assert!(low == EXPECTED_LOW);
    }
}
