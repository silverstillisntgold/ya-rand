//! Approximation of PI using the monte carlo method,
//! for both `f32` and `f64` types.

use std::f32::consts::PI as f32_PI;
use std::f64::consts::PI as f64_PI;
use ya_rand::*;

const ITERATIONS: u64 = 1 << 24;

fn main() {
    let mut rng = new_rng();
    for _ in 0..3 {
        test_f32(&mut rng);
        test_f64(&mut rng);
        println!();
    }
}

fn test_f32(rng: &mut ShiroRng) {
    let mut in_circle: u64 = 0;
    for _ in 0..ITERATIONS {
        let x = rng.f32();
        let y = rng.f32();
        let distance = (x * x) + (y * y);
        if distance <= 1.0 {
            in_circle += 1;
        }
    }

    let simulated = 4.0 * (in_circle as f32) / (ITERATIONS as f32);
    println!("f32 const: {}", f32_PI);
    println!("Simulated: {}", simulated);
    println!(
        "Delta between const and simulated π: {}",
        (f32_PI - simulated).abs()
    );
}

fn test_f64(rng: &mut ShiroRng) {
    let mut in_circle: u64 = 0;
    for _ in 0..ITERATIONS {
        let x = rng.f64();
        let y = rng.f64();
        let distance = (x * x) + (y * y);
        if distance <= 1.0 {
            in_circle += 1;
        }
    }

    let simulated = 4.0 * (in_circle as f64) / (ITERATIONS as f64);
    println!("f64 const: {}", f64_PI);
    println!("Simulated: {}", simulated);
    println!(
        "Delta between const and simulated π: {}",
        (f64_PI - simulated).abs()
    );
}
