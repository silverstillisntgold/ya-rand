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
    let t1 = time_in_nanos(move || {
        v.iter_mut().for_each(|v| {
            v.write(rng.random::<u64>());
        });
        black_box(v);
    });

    let mut v = Box::new_uninit_slice(ITERATIONS);
    let mut rng = fastrand::Rng::with_seed(fastrand::u64(..));
    let t2 = time_in_nanos(move || {
        v.iter_mut().for_each(|v| {
            v.write(rng.u64(..));
        });
        black_box(v);
    });

    let mut v = Box::new_uninit_slice(ITERATIONS);
    let mut rng = new_rng();
    let t3 = time_in_nanos(move || {
        v.iter_mut().for_each(|v| {
            v.write(rng.u64());
        });
        black_box(v);
    });

    let mut v = Box::new_uninit_slice(ITERATIONS);
    let t4 = time_in_nanos(move || {
        v.iter_mut().for_each(|v| {
            v.write(rand::random::<u64>());
        });
        black_box(v);
    });

    let mut v = Box::new_uninit_slice(ITERATIONS);
    let t5 = time_in_nanos(move || {
        v.iter_mut().for_each(|v| {
            v.write(fastrand::u64(..));
        });
        black_box(v);
    });

    let mut v = Box::new_uninit_slice(ITERATIONS);
    let t6 = time_in_nanos(move || {
        v.par_iter_mut().for_each(|v| {
            v.write(rand::random::<u64>());
        });
        black_box(v);
    });

    let mut v = Box::new_uninit_slice(ITERATIONS);
    let t7 = time_in_nanos(move || {
        v.par_iter_mut().for_each(|v| {
            v.write(fastrand::u64(..));
        });
        black_box(v);
    });

    println!(
        "Time to fill a slice with {} values (avg per value) in nanoseconds:\n\
         ----------------------------------------------------------------\n\
         Sequential (local) `rand` average time:     {:>5.2}\n\
         Sequential (local) `fastrand` average time: {:>5.2}\n\
         Sequential (local) `ya-rand` average time:  {:>5.2} <-- You are here\n\
         \n\
         Sequential (TLS)   `rand` average time:     {:>5.2}\n\
         Sequential (TLS)   `fastrand` average time: {:>5.2}\n\
         \n\
         Parallel   (TLS)   `rand` average time:     {:>5.2}\n\
         Parallel   (TLS)   `fastrand` average time: {:>5.2}\n\
         ----------------------------------------------------------------\n",
        ITERATIONS, t1, t2, t3, t4, t5, t6, t7
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
