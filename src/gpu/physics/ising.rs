use bytemuck::bytes_of;
use gpu_random::philox::Philox4x32;
use kernel::IsingCtx;
use wgpu::util::DeviceExt;

use crate::{error::WGPUError, gpu::pipeline::Pipeline};

pub struct IsingPipeline {
    ctx: IsingCtx,
    ctx_buffer: wgpu::Buffer,
    reset_pipeline: Pipeline,
    step_pipeline: Pipeline,
    len: [u32; 2],
}

impl IsingPipeline {
    pub async fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        shader_module: &wgpu::ShaderModule,
        ctx: IsingCtx,
        seed: u128,
    ) -> Result<Self, WGPUError> {
        let ctx_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Ising ctx buffer"),
            contents: bytes_of(&ctx),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let count = (ctx.width * ctx.height) as usize;

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
            ctx,
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
            len: [ctx.width, ctx.height],
        };
        p.reset(device, queue).await?;
        Ok(p)
    }
    pub async fn update_ctx(
        &mut self,
        queue: &wgpu::Queue,
        temperature: f32,
        chemical_potential: f32,
    ) {
        self.ctx.temperature = temperature;
        self.ctx.chemical_potential = chemical_potential;
        queue.write_buffer(&self.ctx_buffer, 0, bytes_of(&self.ctx));
    }
    async fn dispatch<const N: usize>(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        commands: [wgpu::CommandBuffer; N],
        pipeline: &Pipeline,
    ) -> Result<(), WGPUError> {
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

        Ok(())
    }
    pub async fn reset(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> Result<(), WGPUError> {
        self.dispatch(device, queue, [], &self.reset_pipeline).await
    }
    pub async fn step(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<(), WGPUError> {
        self.dispatch(device, queue, [], &self.step_pipeline).await
    }
}
