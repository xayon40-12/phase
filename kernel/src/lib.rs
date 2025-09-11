#![no_std]

use bytemuck::{Pod, Zeroable};
use spirv_std::{
    glam::{UVec3, Vec2, Vec4, vec4},
    spirv,
};

use gpu_random::{GPURng, philox::Philox4x32};

#[allow(unused_imports)]
use num::Float;

/// Struct which stores the size of the system, the temperature and external field strength.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct IsingCtx {
    pub width: u32,
    pub height: u32,
    pub temperature: f32,
    pub external_field: f32,
}

/// Reset the state by randomizing the value in each cells.
#[spirv(compute(threads(1)))]
pub fn ising_reset(
    #[spirv(global_invocation_id)] gid: UVec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ising: &IsingCtx,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] vals: &mut [f32],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] rngs: &mut [Philox4x32],
) {
    let ix = gid.x as usize;
    let iy = gid.y as usize;
    let i = ix + ising.width as usize * iy;
    vals[i] = 1.0 - 2.0 * rngs[i].next_uniform().round();
}

/// Compute shader for the [Ising model](https://en.wikipedia.org/wiki/Ising_model) which compute a new random candidate in each cells and keep it with a probability depending on the energy of both old and candidate states.
#[spirv(compute(threads(1)))]
pub fn ising_step(
    #[spirv(global_invocation_id)] gid: UVec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ising: &IsingCtx,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] vals: &[f32],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] new_vals: &mut [f32],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 3)] rngs: &mut [Philox4x32],
) {
    let ix = gid.x as usize;
    let iy = gid.y as usize;
    let t = ising.temperature;
    let c = ising.external_field;
    let w = ising.width as usize;
    let h = ising.height as usize;
    let i = ix + w * iy;
    let il = ((ix + w - 1) % w) + w * iy;
    let ir = ((ix + 1) % w) + w * iy;
    let iu = ix + w * ((iy + 1) % h);
    let id = ix + w * ((iy + h - 1) % h);

    let v = vals[i];
    let vc = 1.0 - 2.0 * rngs[i].next_uniform().round(); // New candidate
    let s = -(vals[il] + vals[ir] + vals[iu] + vals[id]);

    let e = v * s - c * v;
    let ec = vc * s - c * vc;

    let r = rngs[i].next_uniform();
    let q = ((e - ec) / t).exp();
    let p = q / (1.0 + q);
    if r < p {
        new_vals[i] = vc;
    } else {
        new_vals[i] = v;
    }
}

/// Fragment shader for the Ising model which shows spin up as blue and spin down as white.
#[spirv(fragment)]
pub fn ising_fragment(
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ising: &IsingCtx,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] vals: &[f32],
    uv: Vec2,
    output: &mut Vec4,
) {
    let w = ising.width as f32;
    let h = ising.height as f32;
    let x = (uv.x * (w - 1.0)) as usize;
    let y = (uv.y * (h - 1.0)) as usize;
    let id = x + ising.width as usize * y;
    let val = vals[id];

    *output = vec4(1.0 - val, 1.0 - val, 1.0, 1.0);
}

/// Simple fragment shader to verify that the uv coordinates are correct by showing them in the red and blue channels.
#[spirv(fragment)]
pub fn square_fragment(uv: Vec2, output: &mut Vec4) {
    *output = vec4(uv.x, uv.y, 0.0, 1.0);
}

/// Simple vertex shader for a square made of a triangle stip. It outputs the uv coordinates in `[0,1]` to be used by the fragment shader.
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
