# YA-Rand: Yet Another Rand

Provides simple and fast pseudo/crypto random number generation.

## Usage

```rust
use ya_rand::*;

// **Correct** instantiation is easy.
// This seeds the RNG using operating system entropy,
// meaning you never have to worry about the quality
// of the initial state of RNG instances.
let mut rng = new_rng();

// Generate a random number with a given upper bound
let bound: u64 = 69;
let val = rng.bound(bound);
assert!(val < bound);

// Generate a random number in a given range
let min: i64 = 69;
let max: i64 = 420;
let val = rng.range(min, max);
assert!(min <= val && val < max);

// Generate a random floating point value
let val = rng.f64();
assert!(0.0 <= val && val < 1.0);

// Generate a random ascii digit:
// '0' - '9' as a utf-8 character
let digit = rng.ascii_digit();
assert!(digit.is_ascii_digit());
```

See https://docs.rs/ya-rand for full documentation.
