use std::num::NonZero;

/// Convenient wrapper for ComputePipeline with default parameters.
pub struct Pipeline {
    pub pipeline: wgpu::ComputePipeline,
    pub bind_group: wgpu::BindGroup,
    pub name: String,
}

impl Pipeline {
    /// Contsruct a ComputePipeline with entry point `name` and a list of `entries` as `(binding, buffer, storage type, dynamic offset)`. A value of `None` for the `storage type` means `Uniform` whereas a value of `Some(read_only)` means a `Storage` buffer with the corresponding `read_only` value.
    pub fn new<const N: usize>(
        device: &wgpu::Device,
        shader_module: &wgpu::ShaderModule,
        name: &str,
        entries: [(u32, &wgpu::Buffer, Option<bool>, Option<u64>); N],
    ) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(&format!("{name} Bind Group Layout")),
            entries: &entries.map(|(binding, _, read_only, has_dynamic_offset)| {
                wgpu::BindGroupLayoutEntry {
                    binding,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: if let Some(read_only) = read_only {
                            wgpu::BufferBindingType::Storage { read_only }
                        } else {
                            wgpu::BufferBindingType::Uniform
                        },
                        has_dynamic_offset: has_dynamic_offset.is_some(),
                        min_binding_size: None,
                    },
                    count: None,
                }
            }),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("{name} Bind Group")),
            layout: &bind_group_layout,
            entries: &entries.map(|(binding, buffer, _, size)| wgpu::BindGroupEntry {
                binding,
                resource: if let Some(size) = size {
                    wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer,
                        offset: 0,
                        size: Some(NonZero::new(size as u64).unwrap()),
                    })
                } else {
                    buffer.as_entire_binding()
                },
            }),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&format!("{name} Pipeline Layout")),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(&format!("{name} Pipeline")),
            layout: Some(&pipeline_layout),
            module: shader_module,
            entry_point: Some(name),
            compilation_options: Default::default(),
            cache: None,
        });
        Pipeline {
            pipeline,
            bind_group,
            name: name.to_string(),
        }
    }
}
