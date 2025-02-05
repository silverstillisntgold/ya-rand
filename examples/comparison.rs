//! Compares the performance of alternate RNG crates
//! when filling a slice with random values.

use rand::{rngs, Rng, SeedableRng};
use rayon::prelude::*;
use std::hint::black_box;
use std::time::Instant;
use ya_rand::*;

const ITERATIONS: usize = 1 << 24;

fn main() {
    let mut v = Box::new_uninit_slice(ITERATIONS);
    let mut rng = rngs::StdRng::from_rng(&mut rand::rng());
    let sequential_local_rand = time_in_nanos(move || {
        v.iter_mut().for_each(|v| {
            v.write(rng.random::<u64>());
        });
        black_box(v);
    });

    let mut v = Box::new_uninit_slice(ITERATIONS);
    let mut rng = fastrand::Rng::with_seed(fastrand::u64(..));
    let sequential_local_fastrand = time_in_nanos(move || {
        v.iter_mut().for_each(|v| {
            v.write(rng.u64(..));
        });
        black_box(v);
    });

    let mut v = Box::new_uninit_slice(ITERATIONS);
    let mut rng = new_rng();
    let sequential_local_yarand = time_in_nanos(move || {
        v.iter_mut().for_each(|v| {
            v.write(rng.u64());
        });
        black_box(v);
    });

    let mut v = Box::new_uninit_slice(ITERATIONS);
    let mut rng = oorandom::Rand64::new(rand::random());
    let sequential_local_oorand = time_in_nanos(move || {
        v.iter_mut().for_each(|v| {
            v.write(rng.rand_u64());
        });
        black_box(v);
    });

    let mut v = Box::new_uninit_slice(ITERATIONS);
    let sequential_tls_rand = time_in_nanos(move || {
        v.iter_mut().for_each(|v| {
            v.write(rand::random::<u64>());
        });
        black_box(v);
    });

    let mut v = Box::new_uninit_slice(ITERATIONS);
    let sequential_tls_fastrand = time_in_nanos(move || {
        v.iter_mut().for_each(|v| {
            v.write(fastrand::u64(..));
        });
        black_box(v);
    });

    let mut v = Box::new_uninit_slice(ITERATIONS);
    let parallel_tls_rand = time_in_nanos(move || {
        v.par_iter_mut().for_each(|v| {
            v.write(rand::random::<u64>());
        });
        black_box(v);
    });

    let mut v = Box::new_uninit_slice(ITERATIONS);
    let parallel_tls_fastrand = time_in_nanos(move || {
        v.par_iter_mut().for_each(|v| {
            v.write(fastrand::u64(..));
        });
        black_box(v);
    });

    println!(
        "Filling a slice with {} values - Average time per value in nanoseconds:\n\
         ----------------------------------------------------------------\n\
         Sequential (local) `rand` average time:     {:>5.2}\n\
         Sequential (local) `fastrand` average time: {:>5.2}\n\
         Sequential (local) `ya-rand` average time:  {:>5.2} <-- You are here\n\
         Sequential (local) `oorandom` average time: {:>5.2}\n\
         \n\
         Sequential (TLS)   `rand` average time:     {:>5.2} <-- How most people use `rand`\n\
         Sequential (TLS)   `fastrand` average time: {:>5.2}\n\
         \n\
         Parallel   (TLS)   `rand` average time:     {:>5.2}\n\
         Parallel   (TLS)   `fastrand` average time: {:>5.2}\n\
         ----------------------------------------------------------------\n",
        ITERATIONS,
        sequential_local_rand,
        sequential_local_fastrand,
        sequential_local_yarand,
        sequential_local_oorand,
        sequential_tls_rand,
        sequential_tls_fastrand,
        parallel_tls_rand,
        parallel_tls_fastrand
    );
}

#[inline(never)]
fn time_in_nanos<F>(op: F) -> f64
where
    F: FnOnce(),
{
    let start = Instant::now();
    op();
    let end = Instant::now();
    let delta = end.duration_since(start).as_secs_f64();
    let time = delta / (ITERATIONS as f64);
    time * 1e9
}
