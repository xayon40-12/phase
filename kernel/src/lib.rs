// #![cfg_attr(target_arch = "spirv", no_std)]
#![no_std]

use bytemuck::{Pod, Zeroable};
#[cfg(target_arch = "spirv")]
use spirv_std::{glam::UVec3, spirv};

use gpu_random::{GPURng, philox::Philox4x32};

#[allow(unused_imports)]
use num::Float;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct IsingCtx {
    pub width: u32,
    pub height: u32,
    pub temperature: f32,
    pub chemical_potential: f32,
}

#[inline]
pub fn ising_reset_stage(ix: usize, iy: usize, ising: &IsingCtx, vals: &mut [f32]) {
    let id = ix + ising.width as usize * iy;
    vals[id] = 0.0;
}
#[cfg(target_arch = "spirv")]
#[spirv(compute(threads(1)))]
pub fn ising_reset(
    #[spirv(global_invocation_id)] gid: UVec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ising: &IsingCtx,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] vals: &mut [f32],
) {
    ising_reset_stage(gid.x as usize, gid.y as usize, ising, vals);
}

#[inline]
pub fn ising_stage(
    ix: usize,
    iy: usize,
    ising: &IsingCtx,
    vals: &mut [f32],
    rngs: &mut [Philox4x32],
) {
    let id = ix + ising.width as usize * iy;
    let r = rngs[id].next_f32([id as u32, 0]);
    vals[id] = r.round();
}
#[cfg(target_arch = "spirv")]
#[spirv(compute(threads(1)))]
pub fn ising(
    #[spirv(global_invocation_id)] gid: UVec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ising: &IsingCtx,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] vals: &mut [f32],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] rngs: &mut [Philox4x32],
) {
    ising_stage(gid.x as usize, gid.y as usize, ising, vals, rngs);
}
