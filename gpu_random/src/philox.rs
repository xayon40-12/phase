use core::f32::consts::PI;
#[allow(unused_imports)]
use num::Float;

use bytemuck::{Pod, Zeroable};

use super::GPURng;

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct Philox4x32 {
    counter: [u32; 4],
    normal: [f32; 2],
    current_u32: u32,
    current_normal: u32,
    rounds: u32,
}

#[inline(always)]
pub fn widening_mul_u32(a: u32, b: u32) -> (u32, u32) {
    let a0 = a & 0xFFFF;
    let a1 = a >> 16;
    let b0 = b & 0xFFFF;
    let b1 = b >> 16;

    let p0 = a0 * b0;
    let p1 = a0 * b1;
    let p2 = a1 * b0;
    let p3 = a1 * b1;

    let lo_low16 = p0 & 0xFFFF;
    let carry_mid = (p0 >> 16) + (p1 & 0xFFFF) + (p2 & 0xFFFF);
    let lo_high16 = carry_mid & 0xFFFF;
    let carry_hi = carry_mid >> 16;

    let lo = (lo_high16 << 16) | lo_low16;
    let hi = (p1 >> 16) + (p2 >> 16) + p3 + carry_hi;
    (lo, hi)
}

#[test]
fn test_widening() {
    let mut phi = Philox4x32::new([0; 4], 7);
    for _ in 0..100000 {
        let a = phi.next_u32([0, 0]);
        let b = phi.next_u32([0, 0]);
        let m = a as u64 * b as u64;
        let (lo, hi) = widening_mul_u32(a, b);
        assert!(m as u32 == lo);
        assert!((m >> 32) as u32 == hi);
    }
}

impl Philox4x32 {
    pub fn new(seed: [u32; 4], rounds: u32) -> Self {
        Philox4x32 {
            counter: seed,
            current_u32: u32::MAX,
            normal: [0.0; 2],
            current_normal: u32::MAX,
            rounds,
        }
    }
    pub fn next(&mut self, mut key: [u32; 2]) {
        let counter = &mut self.counter;
        for _ in 0..self.rounds {
            let (lo0, hi0) = widening_mul_u32(0xD2511F53u32, counter[0]);
            let (lo1, hi1) = widening_mul_u32(0xCD9E8D57u32, counter[2]);
            counter[0] = hi1 ^ key[0] ^ counter[1];
            counter[1] = lo1;
            counter[2] = hi0 ^ key[1] ^ counter[3];
            counter[3] = lo0;
            key[0] = key[0].wrapping_add(0x9E3779B9);
            key[1] = key[1].wrapping_add(0xBB67AE85);
        }
        self.current_u32 = 0;
    }
}

impl GPURng for Philox4x32 {
    fn next_u32(&mut self, key: [u32; 2]) -> u32 {
        if self.current_u32 > 3 {
            self.next(key);
        }
        let val = self.counter[self.current_u32 as usize];
        self.current_u32 += 1;
        val
    }
    fn next_f32(&mut self, key: [u32; 2]) -> f32 {
        let exp = 0x3f800000;
        let mask = 0x007fffff;
        f32::from_bits(exp | (self.next_u32(key) & mask)) - 1.0
    }
    fn next_normal(&mut self, key: [u32; 2], mu: f32, sigma: f32) -> f32 {
        if self.current_normal > 1 {
            let u1 = self.next_f32(key);
            let u2 = self.next_f32(key);
            let sqrtln2u1 = (-2.0 * u1.ln()).sqrt();
            let pi2u2 = 2.0 * PI * u2;
            let n1 = sqrtln2u1 * pi2u2.cos();
            let n2 = sqrtln2u1 * pi2u2.sin();
            self.normal = [n1, n2];
            self.current_normal = 0;
        }
        let n = self.normal[self.current_normal as usize];
        self.current_normal += 1;
        mu + sigma * n
    }
}

#[test]
pub fn test_philox_normal() {
    let mut phi = Philox4x32::new([0; 4], 7);
    let mut m1 = 0.0;
    let mut m2 = 0.0;
    let mu = 17.3;
    let sigma = 12.1;
    let count = 10000;
    for _ in 0..count {
        let n = phi.next_normal([0, 0], mu, sigma);
        m1 += n;
        m2 += n * n;
    }
    let inv_count = (count as f32).recip();
    m1 *= inv_count;
    m2 *= inv_count;

    let r_mu = m1;
    let r_sigma = (m2 - m1 * m1).sqrt();
    // println!("{r_mu} {r_sigma}");
    let rel =
        |a: f32, b: f32| (a - b).abs() / (a.abs().max(b.abs()) + f32::EPSILON) < inv_count.sqrt();
    assert!(rel(mu, r_mu));
    assert!(rel(sigma, r_sigma));
}
