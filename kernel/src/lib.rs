#![no_std]

use bytemuck::{Pod, Zeroable};
use spirv_std::{
    glam::{UVec3, Vec2, Vec4, vec4},
    spirv,
};

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

#[spirv(compute(threads(1)))]
pub fn ising_reset(
    #[spirv(global_invocation_id)] gid: UVec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ising: &IsingCtx,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] vals: &mut [f32],
) {
    let ix = gid.x as usize;
    let iy = gid.y as usize;
    let id = ix + ising.width as usize * iy;
    vals[id] = 0.0;
}

#[spirv(compute(threads(1)))]
pub fn ising_step(
    #[spirv(global_invocation_id)] gid: UVec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ising: &IsingCtx,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] vals: &mut [f32],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] rngs: &mut [Philox4x32],
) {
    let ix = gid.x as usize;
    let iy = gid.y as usize;
    let id = ix + ising.width as usize * iy;
    let r = rngs[id].next_f32([id as u32, 0]);
    vals[id] = r * (1.0 + ising.temperature.cos()) * 0.5;
}

#[spirv(fragment)]
pub fn ising_fragment(
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ising: &IsingCtx,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] vals: &[f32],
    uv: Vec2,
    output: &mut Vec4,
) {
    let w = ising.width as f32;
    let h = ising.height as f32;
    let id = (uv.x * (w - 1.0) + w * (h - 1.0) * uv.y) as usize;
    let val = vals[id];

    *output = vec4(uv.x, uv.y, val, 1.0);
}

#[spirv(fragment)]
pub fn square_fragment(uv: Vec2, output: &mut Vec4) {
    *output = vec4(uv.x, uv.y, 0.0, 1.0);
}

#[spirv(vertex)]
pub fn square_vertex(
    #[spirv(vertex_index)] vert_id: i32,
    #[spirv(position)] out_pos: &mut Vec4,
    uv: &mut Vec2,
) {
    uv.x = (vert_id & 1) as f32;
    uv.y = ((vert_id >> 1) & 1) as f32;
    *out_pos = vec4(uv.x * 2.0 - 1.0, uv.y * 2.0 - 1.0, 0.0, 1.0);
}
