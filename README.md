# YA-Rand: Yet Another Rand

Simple and fast pseudo/crypto random number generation.

## Performance considerations

The backing CRNG uses compile-time dispatch, so you'll only get the fastest implementation available to the
machine if rust knows what kind of machine to compile for.

Your best bet is to configure your global .cargo/config.toml with `rustflags = ["-Ctarget-cpu=native"]`
beneath the `[target.<your target triple here>]` directive.

If you know the [x86 feature level] of the processor that will be executing your binaries,
it maybe be better to instead configure this directive at the crate level.

[x86 feature level]: https://en.wikipedia.org/wiki/X86-64#Microarchitecture_levels

## But why?

Because [`rand`] is very cool and extremely powerful, but kind of an enormous fucking pain in the ass
to use, and it's far too large and involved for someone who just needs to flip a coin once every few
loop iterations. But if you're doing some crazy black magic numerical sorcery, it almost certainly
has something you can use to complete your spell. Don't be afraid to go there if you need to.

Other crates, like [`fastrand`], [`tinyrand`], or [`oorandom`], fall somewhere between "I'm not sure I trust
the backing RNG" (state size is too small or algorithm is iffy) and "this API is literally just
`rand` but far less powerful". I wanted something easy to use, but also fast and statistically robust.

So here we are.

[`fastrand`]: https://crates.io/crates/fastrand
[`oorandom`]: https://crates.io/crates/oorandom
[`rand`]: https://crates.io/crates/rand
[`tinyrand`]: https://crates.io/crates/tinyrand

## Usage

"How do I access the thread-local RNG?" There isn't one, and unless Rust improves the performance and
ergonomics of the TLS implementation, there probably won't ever be. Create a local instance when and
where you need one and use it while you need it. If you need an RNG to stick around for a while, passing
it between functions or storing it in structs is a perfectly valid solution.

Here are a few examples to get you started, see https://docs.rs/ya-rand for full documentation and more examples.

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
