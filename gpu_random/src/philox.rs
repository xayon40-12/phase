#[allow(unused_imports)]
use num::Float;

use bytemuck::{Pod, Zeroable};

use crate::widening_mul::widening_mul_u32;

use super::GPURng;

/// Philox counter based random number generator from the Random123 paper:
///
/// John K. Salmon, Mark A. Moraes, Ron O. Dror, and David E. Shaw. 2011. Parallel random numbers: as easy as 1, 2, 3. In Proceedings of 2011 International Conference for High Performance Computing, Networking, Storage and Analysis (SC '11). Association for Computing Machinery, New York, NY, USA, Article 16, 1–12. <https://doi.org/10.1145/2063384.2063405>
#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct Philox4x32 {
    counter: [u32; 4],
    normal: [f32; 2],
    current_u32: u32,
    current_normal: u32,
    key: [u32; 2],
    rounds: u32,
}

impl Philox4x32 {
    /// Create a Philox4x32 with an initial `seed` and `key`. The `key` allows to have many independent streams of random numbers for a same initial `seed`.
    ///
    /// NOTE: This method cannot be called in a WebGPU as it u128 and u64 are not available. Use [Pilox4x32::new_u32] instead.
    pub fn new(seed: u128, key: u64) -> Self {
        Self::new_u32(unsafe { core::mem::transmute(seed) }, unsafe {
            core::mem::transmute(key)
        })
    }
    /// Create a Philox4x32 with an initial `seed` and `key`. The `key` allows to have many independent streams of random numbers for a same initial `seed`.
    pub fn new_u32(seed: [u32; 4], key: [u32; 2]) -> Self {
        Philox4x32 {
            counter: seed,
            current_u32: u32::MAX,
            normal: [0.0; 2],
            key,
            current_normal: u32::MAX,
            rounds: 7,
        }
    }
    /// Set a different number of rounds used by the Philox algorithm.
    pub fn with_rounds(mut self, rounds: u32) -> Self {
        self.rounds = rounds;
        self
    }
    /// Set a fifferent value for the `key`.
    pub fn set_key(&mut self, key: [u32; 2]) {
        self.key = key;
    }
    /// Perform the Philox algorithm once on the counters.
    fn next(&mut self) {
        let mut key = self.key;
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
    fn next_u32(&mut self) -> u32 {
        if self.current_u32 > 3 {
            self.next();
        }
        let val = self.counter[self.current_u32 as usize];
        self.current_u32 += 1;
        val
    }
    fn next_normal(&mut self, mu: f32, sigma: f32) -> f32 {
        if self.current_normal > 1 {
            self.normal = self.next_normal_pair();
            self.current_normal = 0;
        }
        let n = self.normal[self.current_normal as usize];
        self.current_normal += 1;
        mu + sigma * n
    }
}

/// Simple test to verify that the random number from [Philox4x32::next_normal] are actually normally distributed.
#[test]
pub fn test_philox_normal() {
    let mut phi = Philox4x32::new(0, 0);
    let mut m1 = 0.0;
    let mut m2 = 0.0;
    let mu = 17.3;
    let sigma = 12.1;
    let count = 10000;
    for _ in 0..count {
        let n = phi.next_normal(mu, sigma);
        m1 += n;
        m2 += n * n;
    }
    let inv_count = (count as f32).recip();
    m1 *= inv_count;
    m2 *= inv_count;

    let r_mu = m1;
    let r_sigma = (m2 - m1 * m1).sqrt();
    let rel =
        |a: f32, b: f32| (a - b).abs() / (a.abs().max(b.abs()) + f32::EPSILON) < inv_count.sqrt();
    assert!(rel(mu, r_mu));
    assert!(rel(sigma, r_sigma));
}
