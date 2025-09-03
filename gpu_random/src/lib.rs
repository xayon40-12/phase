#![no_std]

use core::f32::consts::PI;
#[allow(unused_imports)]
use num::Float;

pub mod philox;

pub trait GPURng: Clone {
    fn next_u32(&mut self, key: [u32; 2]) -> u32;
    /// Return a uniform number in [0,1)
    fn next_f32(&mut self, key: [u32; 2]) -> f32;
    fn next_uniform(&mut self, key: [u32; 2], min: f32, max: f32) -> f32 {
        min + (max - min) * self.next_f32(key)
    }
    fn next_normal(&mut self, key: [u32; 2], mu: f32, sigma: f32) -> f32 {
        let u1 = self.next_f32(key);
        let u2 = self.next_f32(key);
        let sqrtln2u1 = (-2.0 * u1.ln()).sqrt();
        let pi2u2 = 2.0 * PI * u2;
        let n1 = sqrtln2u1 * pi2u2.cos();
        // let n2 = sqrtln2u1 * pi2u2.sin();
        mu + sigma * n1
    }
}
