# YA-Rand: Yet Another Rand

Provides simple and fast pseudo/crypto random number generation.

## Usage

These are a few straightforward examples to get you started.

```rust
use ya_rand::*;

// **Correct** instantiation is very easy.
// This seeds the RNG using operating system entropy,
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
```

See https://docs.rs/ya-rand/latest/ya_rand/ for full documentation.
