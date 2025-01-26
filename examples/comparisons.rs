use core::hint::black_box;
use rand::{Rng, SeedableRng};
use rayon::prelude::*;
use std::time::Instant;
use ya_rand::*;

const ITERATIONS: usize = 1 << 26;
const NANO: f64 = 1e9;

fn main() {
    let mut v = Box::new_uninit_slice(ITERATIONS);
    // Fucking dogshit API what am I looking at
    let mut rng = rand::rngs::StdRng::from_rng(rand::thread_rng()).unwrap();
    let t1 = time_op(move || {
        v.iter_mut().for_each(|v| {
            v.write(rng.gen::<u64>());
        });
        black_box(v);
    });

    let mut v = Box::new_uninit_slice(ITERATIONS);
    let mut rng = fastrand::Rng::new();
    let t2 = time_op(move || {
        v.iter_mut().for_each(|v| {
            v.write(rng.u64(..));
        });
        black_box(v);
    });

    let mut v = Box::new_uninit_slice(ITERATIONS);
    let mut rng = ShiroRng::new();
    let t3 = time_op(move || {
        v.iter_mut().for_each(|v| {
            v.write(rng.u64());
        });
        black_box(v);
    });

    let mut v = Box::new_uninit_slice(ITERATIONS);
    let t4 = time_op(move || {
        v.iter_mut().for_each(|v| {
            v.write(rand::random::<u64>());
        });
        black_box(v);
    });

    let mut v = Box::new_uninit_slice(ITERATIONS);
    let t5 = time_op(move || {
        v.iter_mut().for_each(|v| {
            v.write(fastrand::u64(..));
        });
        black_box(v);
    });

    let mut v = Box::new_uninit_slice(ITERATIONS);
    let t6 = time_op(move || {
        v.par_iter_mut().for_each(|v| {
            v.write(rand::random::<u64>());
        });
        black_box(v);
    });

    let mut v = Box::new_uninit_slice(ITERATIONS);
    let t7 = time_op(move || {
        v.par_iter_mut().for_each(|v| {
            v.write(fastrand::u64(..));
        });
        black_box(v);
    });

    println!(
        "Sequential (local) `rand` average time:     {:>5.2}\n\
         Sequential (local) `fastrand` average time: {:>5.2}\n\
         Sequential (local) `ya-rand` average time:  {:>5.2} <-- You are here\n\
         \n\
         Sequential (TLS)   `rand` average time:     {:>5.2}\n\
         Sequential (TLS)   `fastrand` average time: {:>5.2}\n\
         \n\
         Parallel   (TLS)   `rand` average time:     {:>5.2}\n\
         Parallel   (TLS)   `fastrand` average time: {:>5.2}",
        t1, t2, t3, t4, t5, t6, t7
    );
    println!();
}

#[inline(never)]
fn time_op<OP>(op: OP) -> f64
where
    OP: FnOnce(),
{
    let start = Instant::now();
    op();
    let end = Instant::now();
    let delta = end.duration_since(start).as_secs_f64();
    let time = delta / (ITERATIONS as f64) * NANO;
    time
}
