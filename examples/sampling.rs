//! Compares performance of two long-running geometric distributions
//! with a `p` value of 0.5.
//!
//! Originally implemented to test an alternative to the `LevelGenerator`
//! in the [`SkipList`](https://crates.io/crates/skiplist) crate.

use std::hint::black_box;
use std::iter;
use std::time::Instant;
use ya_rand::*;

const ITERATIONS: usize = 1 << 24;

fn main() {
    let p = black_box(0.5);
    let mut basic_time = Vec::with_capacity(420);
    let mut advanced_time = Vec::with_capacity(420);

    for i in 16..=32 {
        let mut basic = Basic::new(i, p);
        let mut advanced = Advanced::new(i);

        let start = Instant::now();
        let basic_avg = avg(|| basic.random());
        let delta = start.elapsed().as_secs_f64();
        basic_time.push(delta);

        let start = Instant::now();
        let advanced_avg = avg(|| advanced.random());
        let delta = start.elapsed().as_secs_f64();
        advanced_time.push(delta);

        println!(
            "total: {} || basic_avg: {:.3} || advanced_avg: {:.3} || delta: {:.4}",
            i,
            basic_avg,
            advanced_avg,
            (basic_avg - advanced_avg).abs()
        );
    }

    let result_basic = basic_time.iter().sum::<f64>() / (basic_time.len() as f64);
    let result_advanced = advanced_time.iter().sum::<f64>() / (advanced_time.len() as f64);
    let speedup = result_basic / result_advanced;
    println!("basic time:    {:.4} seconds", result_basic);
    println!("advanced time: {:.4} seconds", result_advanced);
    println!("speedup:       {:.2} seconds", speedup);
    println!();
}

#[inline(never)]
fn avg<F>(f: F) -> f64
where
    F: FnMut() -> usize,
{
    iter::repeat_with(f)
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
    total_inclusive: i32,
    p: f64,
    rng: ShiroRng,
}

impl Basic {
    fn new(total: usize, p: f64) -> Self {
        assert!(total != 0);
        assert!(p > 0.0);
        assert!(p < 1.0);
        Self {
            total,
            total_inclusive: total.try_into().unwrap(),
            p,
            rng: ShiroRng::new(),
        }
    }
}

impl LevelGenerator for Basic {
    fn random(&mut self) -> usize {
        // Invert the CDF of the truncated geometric distribution:
        //
        //   CDF(n) = (q^n - 1) / (q^t - 1)
        //
        // where t is the _exclusive_ upper bound (i.e., total + 1).
        //
        // Solving for n given a uniform variate u in [0, 1]:
        //
        //   n = floor( log_q( 1 + (q^t - 1) * u ) )
        //
        // where q = 1 - p and t is the total number of levels.
        let u = self.rng.f64();
        ((1.0 + (self.p.powi(self.total_inclusive) - 1.0) * u)
            .log(self.p)
            .floor() as usize)
            // When q^total underflows to 0.0 due to floating-point precision,
            // the formula can produce values > total.  This ensures that we
            // never return a level greater than total.
            .min(self.total)

        // Old example.
        //
        // let mut h = 0;
        // let mut x = self.p;
        // let f = self.rng.f64_nonzero();
        // while x > f && h + 1 < self.total {
        //     h += 1;
        //     x *= self.p;
        // }
        // h
    }
}

struct Advanced {
    total: usize,
    rng: ShiroRng,
}

impl Advanced {
    fn new(total: usize) -> Self {
        assert!(total != 0);
        assert!(total <= 64);
        Self {
            total,
            rng: ShiroRng::new(),
        }
    }
}

impl LevelGenerator for Advanced {
    fn random(&mut self) -> usize {
        // Number of failures before success
        let height = self.rng.u64().trailing_zeros() as usize;
        height.min(self.total - 1)
    }
}
