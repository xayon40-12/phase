use std::sync::Arc;

use bytemuck::bytes_of;
use gpu_random::philox::Philox4x32;
use kernel::IsingCtx;
use wgpu::{Buffer, util::DeviceExt};

use crate::{gpu::pipeline::Pipeline, simulation::atomic_f32::AtomicF32};

use super::{Physics, WGPUInfo};

pub struct IsingPipeline {
    ctx_buffer: Buffer,
    reset_pipeline: Pipeline,
    step_pipeline: Pipeline,
    vals_buffer: Buffer,
    width: u32,
    height: u32,
    temperature: Arc<AtomicF32>,
    chemical_potential: Arc<AtomicF32>,
    len: [u32; 2],
}

impl IsingPipeline {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        shader_module: &wgpu::ShaderModule,
        seed: u128,
        width: u32,
        height: u32,
        temperature: Arc<AtomicF32>,
        chemical_potential: Arc<AtomicF32>,
    ) -> Self {
        let ctx = IsingCtx {
            width,
            height,
            temperature: temperature.load(),
            chemical_potential: chemical_potential.load(),
        };
        let ctx_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Ising ctx buffer"),
            contents: bytes_of(&ctx),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let count = (width * height) as usize;

        let vals_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Ising vals buffer"),
            size: count as u64 * size_of::<f32>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let rngs = vec![Philox4x32::new(unsafe { std::mem::transmute(seed) }, 7); count];
        let rngs_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Ising rngs buffer"),
            contents: bytemuck::cast_slice(&rngs),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let p = IsingPipeline {
            reset_pipeline: Pipeline::new(
                device,
                shader_module,
                "ising_reset",
                [
                    (0, &ctx_buffer, None, None),
                    (1, &vals_buffer, Some(false), None),
                ],
            ),
            step_pipeline: Pipeline::new(
                device,
                shader_module,
                "ising_step",
                [
                    (0, &ctx_buffer, None, None),
                    (1, &vals_buffer, Some(false), None),
                    (2, &rngs_buffer, Some(false), None),
                ],
            ),
            ctx_buffer,
            vals_buffer,
            width,
            height,
            temperature,
            chemical_potential,
            len: [ctx.width, ctx.height],
        };
        p.reset(device, queue);
        p
    }
    fn dispatch<const N: usize>(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        commands: [wgpu::CommandBuffer; N],
        pipeline: &Pipeline,
    ) {
        // Encode commands for this single pass
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some(&format!("{} Encoder", pipeline.name)),
        });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some(&format!("{} Pass", pipeline.name)),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(&pipeline.pipeline);
            compute_pass.set_bind_group(0, &pipeline.bind_group, &[]);

            compute_pass.dispatch_workgroups(self.len[0], self.len[1], 1);
        }

        queue.submit(commands.into_iter().chain(Some(encoder.finish())));
        let _ = device.poll(wgpu::MaintainBase::Wait);
    }
    pub fn reset(&self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.dispatch(device, queue, [], &self.reset_pipeline)
    }
    pub fn step(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.dispatch(device, queue, [], &self.step_pipeline)
    }
}

impl Physics for IsingPipeline {
    fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let ctx = IsingCtx {
            width: self.width,
            height: self.height,
            temperature: self.temperature.load(),
            chemical_potential: self.chemical_potential.load(),
        };
        queue.write_buffer(&self.ctx_buffer, 0, bytes_of(&ctx));
        self.step(device, queue)
    }
    fn wgpu_info(&self) -> WGPUInfo {
        (
            "ising_fragment",
            vec![(0, &self.ctx_buffer, true), (1, &self.vals_buffer, false)],
        )
    }
}
