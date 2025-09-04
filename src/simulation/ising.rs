use std::sync::Arc;

use crate::gpu::physics::ising::IsingPipeline;

use super::{Parameter, Simulation, UpadeParameter, atomic_f32::AtomicF32};

pub struct Ising {
    temperature: Arc<AtomicF32>,
    chemical_potential: Arc<AtomicF32>,
}

impl Ising {
    pub fn new() -> Self {
        Ising {
            temperature: Arc::new(AtomicF32::new(0.57)),
            chemical_potential: Arc::new(AtomicF32::new(-2.0)),
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
                range: 1e-1..=1e0,
            },
            Parameter::Slider {
                tag: "C",
                value: self.chemical_potential.load(),
                logarithmic: false,
                range: -4.0..=0.0,
            },
        ]
    }
    fn update_parameter(&mut self, update: UpadeParameter) {
        match update {
            UpadeParameter::Slider { tag, value } => match tag {
                "T" => self.temperature.store(value),
                "C" => self.chemical_potential.store(value),
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
            Arc::clone(&self.chemical_potential),
        ))
    }
}
