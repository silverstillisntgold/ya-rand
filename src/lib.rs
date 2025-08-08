/*!
# YA-Rand: Yet Another Rand

Simple and fast pseudo/crypto random number generation.

## Performance considerations

The backing CRNG uses compile-time dispatch, so you'll only get the fastest implementation available to the
machine if rustc knows what kind of machine to compile for.
If you know the [x86 feature level] of the processor that will be running your binaries, tell rustc to
target that feature level. On Windows, this means adding `RUSTFLAGS=-C target-cpu=<level>` to your system
variables in System Properties -> Advanced -> Environment Variables. You can also manually toggle this for
a single cmd-prompt instance using the [`set`] command. On Unix-based systems the process should be similar.
If you're only going to run the final binary on your personal machine, replace `<level>` with `native`.

If you happen to be building with a nightly toolchain, and for a machine supporting AVX512, the **nightly**
feature provides an AVX512F implementation of the backing ChaCha algorithm.

[x86 feature level]: https://en.wikipedia.org/wiki/X86-64#Microarchitecture_levels
[`set`]: https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/set_1

## But why?

Because `rand` is very cool and extremely powerful, but kind of an enormous fucking pain in the ass
to use, and it's far too large and involved for someone who just needs to flip a coin once
every few minutes. But if you're doing some crazy black magic computational sorcery, it almost
certainly has something you can use to complete your spell.

Other crates, like `fastrand`, `tinyrand`, or `oorandom`, fall somewhere between "I'm not sure I trust
the backing RNG" (state size is too small or algorithm is iffy) and "this API is literally just
`rand` but far less powerful". I wanted something easy, but also fast and statistically robust.

So here we are.

## Usage

Glob import the contents of the library and use [`new_rng`] to create new RNGs wherever
you need them. Then call whatever method you require on those instances. All methods available
are directly accessible through any generator instance via the dot operator, and are named
in a way that should make it easy to quickly identify what you need. See below for a few examples.

If you need cryptographic security, [`new_rng_secure`] will provide you with a [`SecureRng`] instance,
suitable for use in secure contexts.

"How do I access the thread-local RNG?" There isn't one, and unless Rust improves the performance and
ergonomics of the TLS implementation, there probably won't ever be. Create a local instance when and
where you need one and use it while you need it. If you need an RNG to stick around for a while, passing
it between functions or storing it in structs is a perfectly valid solution.

```
use ya_rand::*;

// **Correct** instantiation is very easy.
// Seeds a PRNG instance using operating system entropy,
// so you never have to worry about the quality of the
// initial state.
let mut rng = new_rng();

// Generate a random number with a given upper bound.
let max: u64 = 420;
let val = rng.bound(max);
assert!(val < max);

// Generate a random number in a given range.
let min: i64 = -69;
let max: i64 = 69;
let val = rng.range(min, max);
assert!(min <= val && val < max);

// Generate a random floating point value.
let val = rng.f64();
assert!(0.0 <= val && val < 1.0);

// Generate a random ascii digit: '0'..='9' as a char.
let digit = rng.ascii_digit();
assert!(digit.is_ascii_digit());

// Seeds a CRNG instance with OS entropy.
let mut secure_rng = new_rng_secure();

// We still have access to all the same methods...
let val = rng.f64();
assert!(0.0 <= val && val < 1.0);

// ...but since the CRNG is secure, we also
// get some nice extras.
// Here, we generate a string of random hexidecimal
// characters (base 16), with the shortest length guaranteed
// to be secure.
use ya_rand_encoding::*;
let s = secure_rng.text::<Base16>(Base16::MIN_LEN).unwrap();
assert!(s.len() == Base16::MIN_LEN);
```

## Features

* **std** -
    Enabled by default, but can be disabled for use in `no_std` environments. Enables normal/exponential
    distributions, error type conversions for getrandom, and the **alloc** feature.
* **alloc** -
    Enabled by default. Normally enabled through **std**, but can be enabled on it's own for use in
    `no_std` environments that provide allocation primitives. Enables random generation of secure
    `String` values when using [`SecureRng`].
* **inline** -
    Marks all [`Generator::u64`] implementations with #\[inline\]. Should generally increase
    runtime performance at the cost of binary size and compile time.
    You'll have to test your specific use case to determine if this feature is worth it for you;
    all the RNGs provided tend to be plenty fast without additional inlining.

## Details

This crate uses the [xoshiro] family for pseudo-random number generators. These generators are
very fast, of [very high statistical quality], and small. They aren't cryptographically secure,
but most users don't need their RNG to be secure, they just need it to be random and fast. The default
generator is xoshiro256++, which should provide a large enough period for most users. The xoshiro512++
generator is also provided in case you need a longer period.

[xoshiro]: https://prng.di.unimi.it/
[very high statistical quality]: https://vigna.di.unimi.it/ftp/papers/ScrambledLinear.pdf

All generators output a distinct `u64` value on each call, and the various methods used for transforming
those outputs into more usable forms are all high-quality and well-understood. Placing an upper bound
on these values uses [Lemire's method]. Both inclusive bounding and range-based bounding are applications
of this method, with a few intermediary steps to adjust the bound and apply shifts as needed.
This approach is unbiased and quite fast, but for very large bounds performance might degrade slightly,
since the algorithm may need to sample the underlying RNG multiple times to get an unbiased result.
But this is just a byproduct of how the underlying algorithm works, and isn't something you should ever be
worried about when using the aforementioned methods, since these resamples are few and far between.
If your bound happens to be a power of 2, always use [`Generator::bits`], since it's nothing more
than a bit-shift of the original `u64` provided by the RNG, and will always be as fast as possible.

Floating point values (besides the normal and exponential distributions) are uniformly distributed,
with all the possible outputs being equidistant within the given interval. They are **not** maximally dense;
if that's something you need, you'll have to generate those values yourself. This approach is very fast, and
endorsed by both [Lemire] and [Vigna] (the author of the RNGs used in this crate). The normal distribution
implementation uses the [Marsaglia polar method], returning pairs of independently sampled `f64` values.
Exponential variates are generated using [this approach].

[Lemire's method]: https://arxiv.org/abs/1805.10941
[Lemire]: https://lemire.me/blog/2017/02/28/how-many-floating-point-numbers-are-in-the-interval-01/
[Vigna]: https://prng.di.unimi.it/#remarks
[Marsaglia polar method]: https://en.wikipedia.org/wiki/Marsaglia_polar_method
[this approach]: https://en.wikipedia.org/wiki/Exponential_distribution#Random_variate_generation

## Security

If you're in the market for secure random number generation, this crate provides a secure generator backed
by a highly optimized ChaCha8 implementation from the [`chachacha`] crate.
It functions identically to the other provided RNGs, but with added functionality that wouldn't be safe to
use on pseudo RNGs. Why only 8 rounds? Because people who are very passionate about cryptography are convinced
that's enough, and I have zero reason to doubt them, nor any capacity to prove them wrong.
See page 14 of the [`Too Much Crypto`] paper if you're interested in the justification.

The security guarantees made to the user are identical to those made by ChaCha as an algorithm. It is up
to you to determine if those guarantees meet the demands of your use case.

I reserve the right to change the backing implementation at any time to another RNG which is at least as secure,
without changing the API or bumping the major/minor version. Realistically, this just means I'm willing to bump
this to ChaCha12 if ChaCha8 is ever compromised.

[`Too Much Crypto`]: https://eprint.iacr.org/2019/1492

## Safety

Generators are seeded using entropy from the underlying OS, and have the potential to fail during creation.
But in practice this is extraordinarily unlikely, and isn't something the end-user should ever worry about.
Modern Windows versions (10 and newer) have a crypto subsystem that will never fail during runtime, and
rustc can trivially remove the failure branch when compiling binaries for those systems.
*/

#![deny(missing_docs)]
#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
mod encoding;

/// Module providing the [`Encoder`](crate::encoding::Encoder) trait and concrete implementations
/// of the [RFC 4648](https://datatracker.ietf.org/doc/html/rfc4648) encoding schemes.
#[cfg(feature = "alloc")]
pub mod ya_rand_encoding {
    pub use super::encoding::*;
}

mod rng;
mod romuquad;
mod romutrio;
#[cfg(feature = "secure")]
mod secure;
mod util;
mod xoshiro256pp;
mod xoshiro512pp;

pub use rng::{Generator, SecureGenerator, SeedableGenerator};
pub use romuquad::RomuQuad;
pub use romutrio::RomuTrio;
#[cfg(feature = "secure")]
pub use secure::SecureRng;
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

/// The recommended way to create new CRNG instances.
///
/// Identical to calling [`SecureRng::new`].
#[cfg(feature = "secure")]
#[inline]
pub fn new_rng_secure() -> SecureRng {
    SecureRng::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::collections::BTreeSet;
    use ya_rand_encoding::*;

    const ITERATIONS: usize = 10007;
    const ITERATIONS_LONG: usize = 1 << 24;

    #[test]
    pub fn ascii_alphabetic() {
        let mut rng = new_rng();
        let mut vals = BTreeSet::new();
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
        let mut vals = BTreeSet::new();
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
        let mut vals = BTreeSet::new();
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
        let mut vals = BTreeSet::new();
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
        let mut vals = BTreeSet::new();
        for _ in 0..ITERATIONS {
            let result = rng.ascii_digit();
            assert!(result.is_ascii_digit());
            vals.insert(result);
        }
        assert!(vals.len() == 10);
    }

    #[test]
    fn text_base64() {
        test_text::<Base64>();
    }

    #[test]
    fn text_base64_url() {
        test_text::<Base64URL>();
    }

    #[test]
    fn text_base62() {
        test_text::<Base62>();
    }

    #[test]
    fn text_base32() {
        test_text::<Base32>();
    }

    #[test]
    fn text_base32_hex() {
        test_text::<Base32Hex>();
    }

    #[test]
    fn text_base16() {
        test_text::<Base16>();
    }

    #[test]
    fn text_base16_lowercase() {
        test_text::<Base16Lowercase>();
    }

    fn test_text<E: Encoder>() {
        let s = new_rng_secure().text::<E>(ITERATIONS).unwrap();
        let distinct_bytes = s.bytes().collect::<BTreeSet<_>>();
        let distinct_chars = s.chars().collect::<BTreeSet<_>>();

        let lengths_are_equal = ITERATIONS == s.len()
            && E::CHARSET.len() == distinct_bytes.len()
            && E::CHARSET.len() == distinct_chars.len();
        assert!(lengths_are_equal);

        let contains_all_values = E::CHARSET.iter().all(|c| distinct_bytes.contains(c));
        assert!(contains_all_values);
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

    #[test]
    fn f64() {
        let mut rng = new_rng();
        for _ in 0..ITERATIONS_LONG {
            let val = rng.f64();
            assert!(0.0 <= val && val < 1.0);
        }
    }

    #[test]
    fn f32() {
        let mut rng = new_rng();
        for _ in 0..ITERATIONS_LONG {
            let val = rng.f32();
            assert!(0.0 <= val && val < 1.0);
        }
    }

    #[test]
    fn f64_nonzero() {
        let mut rng = new_rng();
        for _ in 0..ITERATIONS_LONG {
            let val = rng.f64_nonzero();
            assert!(0.0 < val && val <= 1.0);
        }
    }

    #[test]
    fn f32_nonzero() {
        let mut rng = new_rng();
        for _ in 0..ITERATIONS_LONG {
            let val = rng.f32_nonzero();
            assert!(0.0 < val && val <= 1.0);
        }
    }

    #[test]
    fn f64_wide() {
        let mut rng = new_rng();
        for _ in 0..ITERATIONS_LONG {
            let val = rng.f64_wide();
            assert!(val.abs() < 1.0);
        }
    }

    #[test]
    fn f32_wide() {
        let mut rng = new_rng();
        for _ in 0..ITERATIONS_LONG {
            let val = rng.f32_wide();
            assert!(val.abs() < 1.0);
        }
    }
}
