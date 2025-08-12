# YA-Rand: Yet Another Rand

Simple and fast pseudo/crypto random number generation.

## Performance considerations

The backing CRNG uses compile-time dispatch, so you'll only get the fastest implementation available to the
machine if rust knows what kind of machine to compile for.

Your best bet is to configure your global .cargo/config.toml with `rustflags = ["-C", "target-cpu=native"]`
beneath the `[build]` directive.

If you know the [x86 feature level] of the processor that will be running your binaries,
it maybe be better to instead configure this directive at the crate level.

[x86 feature level]: https://en.wikipedia.org/wiki/X86-64#Microarchitecture_levels

## Usage

Here are a few examples to get you started,
see https://docs.rs/ya-rand for full documentation and more examples.

```rust
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
use ya_rand::encoding::*;
let s = secure_rng.text::<Base16>(Base16::MIN_LEN);
assert!(s.len() == Base16::MIN_LEN);
```
