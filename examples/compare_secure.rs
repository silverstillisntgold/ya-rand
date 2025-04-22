//! Compares the performance of various CRNG crates
//! when filling a large slice with random values.

use chacha20::{ChaCha8Rng, rand_core::RngCore};
use rand::{SeedableRng, rngs::StdRng};
use std::hint::black_box;
use std::time::Instant;
use ya_rand::*;

const ITERATIONS: usize = 1 << 24;

fn main() {
    let mut v = Box::new_uninit_slice(ITERATIONS);
    let mut rng = SecureStdRng::new();
    let rand = time_in_nanos(move || {
        v.iter_mut().for_each(|v| {
            v.write(rng.u64());
        });
        black_box(v);
    });

    let mut v = Box::new_uninit_slice(ITERATIONS);
    let mut rng = SecureChaCha20::new();
    let chacha20 = time_in_nanos(move || {
        v.iter_mut().for_each(|v| {
            v.write(rng.u64());
        });
        black_box(v);
    });

    let mut v = Box::new_uninit_slice(ITERATIONS);
    let mut rng = new_rng_secure();
    let ya_rand_secure = time_in_nanos(move || {
        v.iter_mut().for_each(|v| {
            v.write(rng.u64());
        });
        black_box(v);
    });

    let mut v = Box::new_uninit_slice(ITERATIONS);
    let mut rng = new_rng();
    let ya_rand = time_in_nanos(move || {
        v.iter_mut().for_each(|v| {
            v.write(rng.u64());
        });
        black_box(v);
    });

    println!(
        "Filling a slice with {} values || Average nanoseconds per value generated:\n\
         (all generators obey the same inlining rules)\n\
         ----------------------------------------------------------------\n\
         `rand` average time:             {:>5.2}\n\
         `chacha20` average time:         {:>5.2}\n\
         `ya-rand` (secure) average time: {:>5.2} <-- You are here\n\
         `ya-rand` average time:          {:>5.2} <-- and here\n\
         ----------------------------------------------------------------\n",
        ITERATIONS, rand, chacha20, ya_rand_secure, ya_rand
    );
}

#[inline(never)]
fn time_in_nanos<F: FnOnce()>(op: F) -> f64 {
    let start = Instant::now();
    op();
    let end = Instant::now();
    let delta = end.duration_since(start).as_secs_f64();
    let time = delta / (ITERATIONS as f64);
    time * 1e9
}

/// Needs a wrapper type because by default `rand` tries to inline
/// ***EVERYTHING***, but this crate is much more conservative with inlining.
/// This isn't intended as a reference benchmark, so it's more useful to
/// see how all the generators compare when on the same playing field.
struct SecureStdRng {
    internal: StdRng,
}

impl YARandGenerator for SecureStdRng {
    fn try_new() -> Result<Self, getrandom::Error> {
        let mut data = <StdRng as SeedableRng>::Seed::default();
        getrandom::fill(&mut data)?;
        let internal = StdRng::from_seed(data);
        Ok(Self { internal })
    }

    #[cfg_attr(not(feature = "inline"), inline(never))]
    #[cfg_attr(feature = "inline", inline(always))]
    fn u64(&mut self) -> u64 {
        self.internal.next_u64()
    }
}

/// Same rationale as `SecureStdRng`.
struct SecureChaCha20 {
    internal: ChaCha8Rng,
}

impl YARandGenerator for SecureChaCha20 {
    fn try_new() -> Result<Self, getrandom::Error> {
        let mut data = <ChaCha8Rng as SeedableRng>::Seed::default();
        getrandom::fill(&mut data)?;
        let internal = ChaCha8Rng::from_seed(data);
        Ok(Self { internal })
    }

    #[cfg_attr(not(feature = "inline"), inline(never))]
    #[cfg_attr(feature = "inline", inline(always))]
    fn u64(&mut self) -> u64 {
        self.internal.next_u64()
    }
}
