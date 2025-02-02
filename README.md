# YA-Rand: Yet Another Rand

Provides simple and fast pseudo/crypto random number generation.

## Usage

```rust
use ya_rand::*;

// Instantiation is that easy
let mut rng = new_rng();

// Generate a bounded random number
let max: u64 = 69;
let val = rng.bound(max);
assert!(val < max);

// Generate a random floating point value
let val = rng.f64();
assert!(0.0 <= val && val < 1.0);

// Generate a random ascii digit
let digit = rng.ascii_digit();
assert!(digit.is_ascii_digit());
```

See https://docs.rs/ya-rand for documentation.
