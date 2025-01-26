use core::hint::black_box;
use std::time::Instant;
use ya_rand::*;

//use rayon::prelude::*;

const ITERATIONS: usize = 1 << 27;

fn main() {
    let mut v = vec![0; ITERATIONS];
    let (_, t1) = time_op(move || {
        v.iter_mut().for_each(|v| *v = rand::random::<u64>());
        black_box(v);
    });

    let mut v = vec![0; ITERATIONS];
    let mut rng = fastrand::Rng::with_seed(ShiroRng::new().u64());
    let (_, t2) = time_op(move || {
        v.iter_mut().for_each(|v| *v = rng.u64(..));
        black_box(v);
    });

    let mut v = vec![0; ITERATIONS];
    let mut rng = ShiroRng::new();
    let (_, t3) = time_op(move || {
        v.iter_mut().for_each(|v| *v = rng.u64());
        black_box(v);
    });

    println!("{} || {} || {}", t1, t2, t3);
}

#[inline(never)]
fn time_op<OP, R>(op: OP) -> (R, f64)
where
    OP: FnOnce() -> R,
{
    let start = Instant::now();
    let r = op();
    let time = get_nanos(start);
    (r, time)
}

#[inline(always)]
fn get_nanos(start: Instant) -> f64 {
    Instant::now().duration_since(start).as_secs_f64() / (ITERATIONS as f64) * 1e9
}
