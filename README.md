# YA-Rand: Yet Another Rand

Simple and fast pseudo/crypto random number generation.

## Performance considerations for users of [`SecureRng`]

The backing CRNG uses compile-time dispatch, so you'll only get the fastest implementation available to the
machine if rustc knows what kind of machine to compile for.
If you know the [x86 feature level] of the processor that will be running your binaries, tell rustc to
target that feature level. On Windows, this means adding `RUSTFLAGS=-C target-cpu=<level>` to your system
variables in System Properties -> Advanced -> Environment Variables. You can also manually toggle this for
a single cmd-prompt instance using the [`set`] command. On Unix-based systems the process should be similar.
If you're only going to run the final binary on your personal machine, replace `<level>` with `native`.

If you happen to be building with a nightly toolchain, and for a machine supporting AVX512, the **nightly**
feature provides an extremely fast AVX512 implementation of the backing ChaCha algorithm.

[`SecureRng`]: https://docs.rs/ya-rand/latest/ya_rand/struct.SecureRng.html
[x86 feature level]: https://en.wikipedia.org/wiki/X86-64#Microarchitecture_levels
[`set`]: https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/set_1

## Windows 10 users on Rust 1.71 or newer

It is ***highly*** recommended that you add `RUSTFLAGS=--cfg windows_raw_dylib` to your path. Currently, the
[`getrandom`] crate that's used to seed RNGs behind the scenes defers its operation to `windows-targets`,
which by default statically links to a 5-12MB lib. Adding the above cfg flag tells it to instead use
the `raw-dylib` feature, which was stabilized in Rust 1.71. This turns `windows-targets` into a small
macro-only library, which improves compile times and decreases binary size for both debug and release builds.

[`getrandom`]: https://docs.rs/getrandom/latest/getrandom/

## Usage

These are just a few examples to get you started.

```rust
use ya_rand::*;

// **Correct** instantiation is very easy.
// Seeds the default PRNG using operating system entropy,
// so you never have to worry about the quality of the
// initial state of RNG instances.
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

// Randomly seeds the CRNG with OS entropy.
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

See https://docs.rs/ya-rand/latest/ya_rand/ for full documentation and more examples.
