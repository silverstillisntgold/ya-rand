//! Compares the performance of various RNG crates when
//! filling a large slice with random values.

use chacha20::ChaCha8Rng;
use rand::{Rng, SeedableRng, rngs::StdRng};
use rayon::prelude::*;
use std::hint::black_box;
use std::time::Instant;
use ya_rand::*;

const ITERATIONS: usize = 1 << 24;

fn main() {
    // Make sure rayon and TLS generators have been initialized.
    let (_, _) = rayon::join(
        || black_box(rand::random::<u64>()),
        || black_box(fastrand::u64(..)),
    );

    let mut v = new_vec();
    let mut rng = SecureStdRng::new();
    let sequential_local_rand = time_in_nanos(move || {
        v.iter_mut().for_each(|v| {
            *v = rng.u64();
        });
        black_box(v);
    });

    let mut v = new_vec();
    let mut rng = fastrand::Rng::with_seed(fastrand::u64(..));
    let sequential_local_fastrand = time_in_nanos(move || {
        v.iter_mut().for_each(|v| {
            *v = rng.u64(..);
        });
        black_box(v);
    });

    let mut v = new_vec();
    let mut rng = new_rng();
    let sequential_local_yarand = time_in_nanos(move || {
        v.iter_mut().for_each(|v| {
            *v = rng.u64();
        });
        black_box(v);
    });

    let mut v = new_vec();
    let mut rng = new_rng_secure();
    let sequential_local_yarand_secure = time_in_nanos(move || {
        v.iter_mut().for_each(|v| {
            *v = rng.u64();
        });
        black_box(v);
    });

    let mut v = new_vec();
    let mut rng = SecureChaCha20::new();
    let sequential_local_chacha20 = time_in_nanos(move || {
        v.iter_mut().for_each(|v| {
            *v = rng.u64();
        });
        black_box(v);
    });

    let mut v = new_vec();
    let mut rng = oorandom::Rand64::new(rand::random());
    let sequential_local_oorand = time_in_nanos(move || {
        v.iter_mut().for_each(|v| {
            *v = rng.rand_u64();
        });
        black_box(v);
    });

    let mut v = new_vec();
    let sequential_tls_rand = time_in_nanos(move || {
        v.iter_mut().for_each(|v| {
            *v = rand::random::<u64>();
        });
        black_box(v);
    });

    let mut v = new_vec();
    let sequential_tls_fastrand = time_in_nanos(move || {
        v.iter_mut().for_each(|v| {
            *v = fastrand::u64(..);
        });
        black_box(v);
    });

    let mut v = new_vec();
    let parallel_local_yarand = time_in_nanos(move || {
        v.par_iter_mut().for_each_init(new_rng, |rng, v| {
            *v = rng.u64();
        });
        black_box(v);
    });

    let mut v = new_vec();
    let parallel_local_yarand_secure = time_in_nanos(move || {
        v.par_iter_mut().for_each_init(new_rng_secure, |rng, v| {
            *v = rng.u64();
        });
        black_box(v);
    });

    let mut v = new_vec();
    let parallel_tls_rand = time_in_nanos(move || {
        v.par_iter_mut().for_each(|v| {
            *v = rand::random::<u64>();
        });
        black_box(v);
    });

    let mut v = new_vec();
    let parallel_tls_fastrand = time_in_nanos(move || {
        v.par_iter_mut().for_each(|v| {
            *v = fastrand::u64(..);
        });
        black_box(v);
    });

    println!(
        "Filling a slice with {} values || Average nanoseconds per value generated:\n\
         ----------------------------------------------------------------\n\
         Sequential (local) `rand`       (secure) average time: {:>5.2}\n\
         Sequential (local) `fastrand` (unsecure) average time: {:>5.2}\n\
         Sequential (local) `ya-rand`  (unsecure) average time: {:>5.2} <-- You are here\n\
         Sequential (local) `ya-rand`    (secure) average time: {:>5.2} <-- and here\n\
         Sequential (local) `chacha20`   (secure) average time: {:>5.2}\n\
         Sequential (local) `oorandom` (unsecure) average time: {:>5.2}\n\
         \n\
         Sequential (TLS)   `rand`       (secure) average time: {:>5.2} <-- How most people use `rand`\n\
         Sequential (TLS)   `fastrand` (unsecure) average time: {:>5.2}\n\
         \n\
         Parallel   (local) `ya-rand`  (unsecure) average time: {:>5.2} <-- You are here\n\
         Parallel   (local) `ya-rand`    (secure) average time: {:>5.2} <-- and here\n\
         \n\
         Parallel   (TLS)   `rand`       (secure) average time: {:>5.2}\n\
         Parallel   (TLS)   `fastrand` (unsecure) average time: {:>5.2}\n\
         ----------------------------------------------------------------\n",
        ITERATIONS,
        sequential_local_rand,
        sequential_local_fastrand,
        sequential_local_yarand,
        sequential_local_yarand_secure,
        sequential_local_chacha20,
        sequential_local_oorand,
        sequential_tls_rand,
        sequential_tls_fastrand,
        parallel_local_yarand,
        parallel_local_yarand_secure,
        parallel_tls_rand,
        parallel_tls_fastrand
    );
}

/// Using a non-zero initializer is crucial, as it ensures
/// our program page-faults the entire allocation before testing
/// of the actual RNG begins. Without this, measurements are
/// very inconsistent and entirely unreliable.
#[inline]
fn new_vec() -> Vec<u64> {
    vec![u64::MAX; ITERATIONS]
}

#[inline(never)]
fn time_in_nanos<F: FnOnce()>(op: F) -> f64 {
    let start = Instant::now();
    op();
    let delta = start.elapsed().as_secs_f64();
    delta / (ITERATIONS as f64) * 1e9
}

/// Needs a wrapper type because by default `rand` tries to inline
/// ***EVERYTHING***, but this crate is much more conservative with inlining.
/// This isn't intended to be a reference benchmark, so it's more useful to
/// see how all the generators compare when on the same playing field.
struct SecureStdRng {
    internal: StdRng,
}

impl Generator for SecureStdRng {
    fn try_new() -> Result<Self, getrandom::Error> {
        let mut data = [0; _];
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

/// Same rationale as [`SecureStdRng`].
struct SecureChaCha20 {
    internal: ChaCha8Rng,
}

impl Generator for SecureChaCha20 {
    fn try_new() -> Result<Self, getrandom::Error> {
        let mut data = [0; _];
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
