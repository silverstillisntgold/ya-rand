# YA-Rand: Yet Another Rand

Provides simple and fast pseudo/crypto random number generation.

## Usage

```rust
use ya_rand::*;

let mut rng = new_rng();
let max: u64 = 69;
// That's all there is to it.
let val: u64 = rng.bound(max);
assert!(val < max);
```

See https://docs.rs/ya-rand for documentation.
