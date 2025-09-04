use egui_wgpu::{CallbackTrait, RenderState};
use wgpu::ShaderModule;

use crate::gpu::physics::Physics;

#[derive(Clone, Copy)]
pub struct RenderSquare {}

impl RenderSquare {
    pub fn new(
        wgpu_render_state: &RenderState,
        shader_module: &ShaderModule,
        physics: Box<dyn Physics>,
    ) -> Self {
        let device = &wgpu_render_state.device;

        let (fragment_entry_point, entries) = physics.wgpu_info();

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Render square bind group layout"),
            entries: &entries
                .iter()
                .cloned()
                .map(|(binding, _, uniform)| wgpu::BindGroupLayoutEntry {
                    binding,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: if uniform {
                            wgpu::BufferBindingType::Uniform
                        } else {
                            wgpu::BufferBindingType::Storage { read_only: true }
                        },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                })
                .collect::<Vec<_>>(),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render square pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render square pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some("square_vertex"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: Some(fragment_entry_point),
                targets: &[Some(wgpu_render_state.target_format.into())],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Render square bind group"),
            layout: &bind_group_layout,
            entries: &entries
                .into_iter()
                .map(|(binding, buffer, _)| wgpu::BindGroupEntry {
                    binding,
                    resource: buffer.as_entire_binding(),
                })
                .collect::<Vec<_>>(),
        });

        // Because the graphics pipeline must have the same lifetime as the egui render pass,
        // instead of storing the pipeline in our `Custom3D` struct, we insert it into the
        // `paint_callback_resources` type map, which is stored alongside the render pass.
        wgpu_render_state
            .renderer
            .write()
            .callback_resources
            .insert(SquareRenderResources {
                pipeline,
                bind_group,
                physics,
            });

        Self {}
    }
}

impl CallbackTrait for RenderSquare {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &egui_wgpu::ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let resources: &mut SquareRenderResources = resources.get_mut().unwrap();
        resources.prepare(device, queue);
        Vec::new()
    }

    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        resources: &egui_wgpu::CallbackResources,
    ) {
        let resources: &SquareRenderResources = resources.get().unwrap();
        resources.paint(render_pass);
    }
}

struct SquareRenderResources {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    physics: Box<dyn Physics>,
}

impl SquareRenderResources {
    fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.physics.update(device, queue);
    }

    fn paint(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw(0..4, 0..1);
    }
}
