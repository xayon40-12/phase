use std::sync::Arc;

use bytemuck::bytes_of;
use gpu_random::philox::Philox4x32;
use instant::Instant;
use kernel::IsingCtx;
use wgpu::{Buffer, CommandEncoder, util::DeviceExt};

use crate::{gpu::pipeline::Pipeline, simulation::atomic_f32::AtomicF32};

use super::{FragmentEntry, FragmentInfo, Physics};

/// Handles the compute pipeline for the Ising model simulation.
pub struct IsingPipeline {
    ctx_buffer: Buffer,
    reset_pipeline: Pipeline,
    step_pipeline: Pipeline,
    vals_buffer: Buffer,
    new_vals_buffer: Buffer,
    width: u32,
    height: u32,
    temperature: Arc<AtomicF32>,
    external_field: Arc<AtomicF32>,
    step_per_frames: usize,
    time_history: [f32; 10],
    current_time: usize,
    time: Instant,
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
        external_field: Arc<AtomicF32>,
    ) -> Self {
        let ctx = IsingCtx {
            width,
            height,
            temperature: temperature.load(),
            external_field: external_field.load(),
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

        let new_vals_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Ising new vals buffer"),
            size: count as u64 * size_of::<f32>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let rngs = (0..count)
            .map(|i| Philox4x32::new(seed, i as u64))
            .collect::<Vec<_>>();
        let rngs_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Ising rngs buffer"),
            contents: bytemuck::cast_slice(&rngs),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let p = IsingPipeline {
            reset_pipeline: Pipeline::new(
                device,
                shader_module,
                "ising_reset",
                [
                    (0, &ctx_buffer, None, None),
                    (1, &vals_buffer, Some(false), None),
                    (2, &rngs_buffer, Some(false), None),
                ],
            ),
            step_pipeline: Pipeline::new(
                device,
                shader_module,
                "ising_step",
                [
                    (0, &ctx_buffer, None, None),
                    (1, &vals_buffer, Some(true), None),
                    (2, &new_vals_buffer, Some(false), None),
                    (3, &rngs_buffer, Some(false), None),
                ],
            ),
            ctx_buffer,
            vals_buffer,
            new_vals_buffer,
            width,
            height,
            temperature,
            external_field,
            step_per_frames: 1,
            time_history: Default::default(),
            current_time: 0,
            time: Instant::now(),
        };
        p.reset(device, queue);
        p
    }
    fn dispatch(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        with_encoder: impl Fn(&mut CommandEncoder),
        repetitions: usize,
        pipeline: &Pipeline,
    ) {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some(&format!("{} Encoder", pipeline.name)),
        });

        for _ in 0..repetitions {
            {
                let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some(&format!("{} Pass", pipeline.name)),
                    timestamp_writes: None,
                });

                compute_pass.set_pipeline(&pipeline.pipeline);
                compute_pass.set_bind_group(0, &pipeline.bind_group, &[]);

                compute_pass.dispatch_workgroups(self.width, self.height, 1);
            }

            with_encoder(&mut encoder);
        }

        queue.submit(Some(encoder.finish()));
        let _ = device.poll(wgpu::MaintainBase::Wait);
    }
    pub fn reset(&self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.dispatch(device, queue, |_| {}, 1, &self.reset_pipeline)
    }
    pub fn step(&mut self, repetitions: usize, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.dispatch(
            device,
            queue,
            |encoder| {
                encoder.copy_buffer_to_buffer(
                    &self.new_vals_buffer,
                    0,
                    &self.vals_buffer,
                    0,
                    self.vals_buffer.size(),
                );
            },
            repetitions,
            &self.step_pipeline,
        )
    }
}

impl Physics for IsingPipeline {
    fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let ctx = IsingCtx {
            width: self.width,
            height: self.height,
            temperature: self.temperature.load(),
            external_field: self.external_field.load(),
        };
        queue.write_buffer(&self.ctx_buffer, 0, bytes_of(&ctx));
        self.step(self.step_per_frames, device, queue);

        // Automatically handle performance by looking at the time taken by an entire frame (aiming for 60 fps). Increase the number of steps per frames if the average time of the 10 last frames is bellow 0.017 (just above 0.016666=1/60), and decrease if the time exceeds 0.017*1.05. The gap between 0.017 and 0.017*1.05 is to avoible oscillations of the number of steps per frames.
        self.time_history[self.current_time] = self.time.elapsed().as_secs_f32();
        self.current_time += 1;
        self.time = Instant::now();
        let len = self.time_history.len();
        if self.current_time == len {
            self.current_time = 0;
            let elapsed = self.time_history.iter().cloned().sum::<f32>() / len as f32;
            let limit = 0.017;
            if elapsed < limit {
                self.step_per_frames = (self.step_per_frames + 1).min(10);
            } else if elapsed > limit * 1.05 {
                self.step_per_frames = (self.step_per_frames - 1).max(1);
            }
        }
    }
    fn wgpu_fragment_info(&self) -> FragmentInfo {
        // The fragment shader kernel to render the value computed by the IsingPipeline is the function located in kernel/src/lib.rs called `ising_fragment`. It takes the context and values so `self.ctx_buffer` and `self.vals_buffer`.
        FragmentInfo {
            fragment_entry_point: "ising_fragment",
            entries: vec![
                FragmentEntry {
                    binding: 0,
                    buffer: &self.ctx_buffer,
                    uniform: true,
                },
                FragmentEntry {
                    binding: 1,
                    buffer: &self.vals_buffer,
                    uniform: false,
                },
            ],
        }
    }
}
