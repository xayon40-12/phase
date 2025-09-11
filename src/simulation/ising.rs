use std::sync::Arc;

use crate::gpu::physics::ising::IsingPipeline;

use super::{Parameter, Simulation, UpadeParameter, atomic_f32::AtomicF32};

/// Bridge between the egui rendering/events and the compute pipeline [IsingPipeline].
pub struct Ising {
    temperature: Arc<AtomicF32>,
    external_field: Arc<AtomicF32>,
}

impl Ising {
    pub fn new() -> Self {
        Ising {
            temperature: Arc::new(AtomicF32::new(2.2691853142)),
            external_field: Arc::new(AtomicF32::new(0.0)),
        }
    }
}

impl Simulation for Ising {
    fn egui_parameters(&self) -> Vec<Parameter> {
        vec![
            Parameter::Slider {
                tag: "T",
                value: self.temperature.load(),
                logarithmic: true,
                range: 1e-1..=1e1,
            },
            Parameter::Slider {
                tag: "h",
                value: self.external_field.load(),
                logarithmic: false,
                range: -2.0..=2.0,
            },
        ]
    }
    fn update_parameter(&mut self, update: UpadeParameter) {
        match update {
            UpadeParameter::Slider { tag, value } => match tag {
                "T" => self.temperature.store(value),
                "h" => self.external_field.store(value),
                _ => {
                    panic!("Unexpected tag in update_parameter: \"{tag}\"")
                }
            },
            _ => {}
        }
    }
    fn physics(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        shader_module: &wgpu::ShaderModule,
        seed: u128,
        width: u32,
        height: u32,
    ) -> Box<dyn crate::gpu::physics::Physics> {
        Box::new(IsingPipeline::new(
            device,
            queue,
            shader_module,
            seed,
            width,
            height,
            Arc::clone(&self.temperature),
            Arc::clone(&self.external_field),
        ))
    }
}
