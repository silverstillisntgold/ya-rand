use core::hint::black_box;
use std::time::Instant;
use ya_rand::*;

const ITERATIONS: u64 = 1 << 26;

#[cfg(feature = "secure")]
type Rng = SecureRng;
#[cfg(not(feature = "secure"))]
type Rng = ShiroRng;

fn main() {
    let mut rng = Rng::new();
    let mut res = black_box(0);
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        res = rng.u64();
    }
    let time = get_nanos(start);
    println!("execution time: {:.2} ns || {}", time, res);
}

#[inline(always)]
fn get_nanos(start: Instant) -> f64 {
    Instant::now().duration_since(start).as_secs_f64() / (ITERATIONS as f64) * 1e9
}
