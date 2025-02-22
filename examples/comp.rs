//! Compares the performance of alternate RNG crates
//! when filling a slice with random values.

use rand::{rngs, Rng, SeedableRng};
use std::hint::black_box;
use std::time::Instant;
use ya_rand::*;

const ITERATIONS: usize = 1 << 24;

fn main() {
    let mut v = Box::new_uninit_slice(ITERATIONS);
    let mut rng = rngs::StdRng::from_rng(&mut rand::rng());
    let rand = time_in_nanos(move || {
        v.iter_mut().for_each(|v| {
            v.write(rng.random::<u64>());
        });
        black_box(v);
    });

    let mut v = Box::new_uninit_slice(ITERATIONS);
    let mut rng = SecureRngLocal::new();
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
         ----------------------------------------------------------------\n\
         `rand` average time:             {:>5.2}\n\
         `chacha20` average time:         {:>5.2}\n\
         `ya-rand` (secure) average time: {:>5.2}\n\
         `ya-rand` average time:          {:>5.2}\n\
         ----------------------------------------------------------------\n",
        ITERATIONS, rand, chacha20, ya_rand_secure, ya_rand
    );
}

#[inline(always)]
fn time_in_nanos<F: FnOnce()>(op: F) -> f64 {
    let start = Instant::now();
    op();
    let end = Instant::now();
    let delta = end.duration_since(start).as_secs_f64();
    let time = delta / (ITERATIONS as f64);
    time * 1e9
}

use chacha20::{rand_core::RngCore, ChaCha8Rng};

#[derive(Debug)]
pub struct SecureRngLocal {
    internal: ChaCha8Rng,
}

impl SecureYARandGenerator for SecureRngLocal {
    #[inline(never)]
    fn fill_bytes(&mut self, dst: &mut [u8]) {
        self.internal.fill_bytes(dst);
    }
}

impl YARandGenerator for SecureRngLocal {
    fn try_new() -> Result<Self, getrandom::Error> {
        const SEED_LEN: usize = 32;
        const STREAM_LEN: usize = 12;
        // Using a combined array so we only need a single syscall.
        let mut data = [0; SEED_LEN + STREAM_LEN];
        getrandom::fill(&mut data)?;
        // Both of these unwraps get optimized out.
        let seed: [u8; SEED_LEN] = data[..SEED_LEN].try_into().unwrap();
        let stream: [u8; STREAM_LEN] = data[SEED_LEN..].try_into().unwrap();
        let mut internal = ChaCha8Rng::from_seed(seed);
        // Randomizing the stream number further decreases the
        // already-low odds of two instances colliding.
        internal.set_stream(stream);
        Ok(Self { internal })
    }

    #[cfg_attr(feature = "inline", inline)]
    fn u64(&mut self) -> u64 {
        self.internal.next_u64()
    }
}
