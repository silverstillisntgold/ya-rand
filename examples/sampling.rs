//! Compares performance of two long-running geometric distributions
//! with a `p` of 0.5.
//!
//! Originally implemented to test an alternative to the `LevelGenerator`
//! in the [`SkipList`](https://crates.io/crates/skiplist) crate.

use std::time::Instant;
use ya_rand::*;

const ITERATIONS: usize = 1 << 24;

fn main() {
    let p = core::hint::black_box(0.5);
    let mut basic_time = Vec::with_capacity(420);
    let mut advanced_time = Vec::with_capacity(420);
    for i in 16..=32 {
        let mut basic = Basic::new(i, p);
        let mut advanced = Advanced::new(i);

        let start = Instant::now();
        let basic_avg = avg(|| basic.random());
        basic_time.push(Instant::now().duration_since(start).as_secs_f64());

        let start = Instant::now();
        let advanced_avg = avg(|| advanced.random());
        advanced_time.push(Instant::now().duration_since(start).as_secs_f64());

        println!(
            "total: {} || basic_avg: {:.4} || advanced_avg: {:.4} || delta: {:.5}",
            i,
            basic_avg,
            advanced_avg,
            (basic_avg - advanced_avg).abs()
        );
    }

    let result_basic = basic_time.iter().sum::<f64>() / (basic_time.len() as f64);
    let result_advanced = advanced_time.iter().sum::<f64>() / (advanced_time.len() as f64);
    println!("basic time: {:.4} seconds", result_basic);
    println!("advanced time: {:.4} seconds", result_advanced);
    println!("speedup: {:.2}", result_basic / result_advanced);
    println!();
}

#[inline(never)]
fn avg<F>(f: F) -> f64
where
    F: FnMut() -> usize,
{
    core::iter::repeat_with(f)
        .take(ITERATIONS)
        .map(|v| v as f64)
        .sum::<f64>()
        / (ITERATIONS as f64)
}

trait LevelGenerator {
    fn random(&mut self) -> usize;
}

struct Basic {
    total: usize,
    p: f64,
    rng: ShiroRng,
}

impl Basic {
    fn new(total: usize, p: f64) -> Self {
        assert!(total != 0);
        assert!(p > 0.0 && p < 1.0);
        Self {
            total,
            p,
            rng: ShiroRng::new(),
        }
    }
}

impl LevelGenerator for Basic {
    fn random(&mut self) -> usize {
        let mut h = 0;
        let mut x = self.p;
        let f = self.rng.f64_nonzero();
        while x > f && h + 1 < self.total {
            h += 1;
            x *= self.p;
        }
        h
    }
}

struct Advanced {
    total: usize,
    rng: ShiroRng,
}

impl Advanced {
    fn new(total: usize) -> Self {
        assert!(total != 0);
        Self {
            total,
            rng: ShiroRng::new(),
        }
    }
}

impl LevelGenerator for Advanced {
    fn random(&mut self) -> usize {
        let height = self.rng.u64().trailing_ones() as usize;
        height.min(self.total - 1)
    }
}
