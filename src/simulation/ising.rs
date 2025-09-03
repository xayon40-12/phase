use kernel::IsingCtx;

use super::{Parameter, Simulation, UpadeParameter};

pub struct Ising {
    ctx: IsingCtx,
}

impl Ising {
    pub fn new(ctx: IsingCtx) -> Self {
        Ising { ctx }
    }
}

impl Simulation for Ising {
    fn reset(&mut self) {
        //TODO
    }

    fn egui_parameters(&self) -> Vec<Parameter> {
        vec![
            Parameter::Slider {
                tag: "T",
                value: self.ctx.temperature,
                logarithmic: false,
                range: 1e-1..=1e1,
            },
            Parameter::Slider {
                tag: "C",
                value: self.ctx.chemical_potential,
                logarithmic: false,
                range: 1e-1..=1e1,
            },
        ]
    }
    fn update_parameter(&mut self, update: UpadeParameter) {
        match update {
            UpadeParameter::Slider { tag, value } => match tag {
                "T" => self.ctx.temperature = value,
                "C" => self.ctx.chemical_potential = value,
                _ => {
                    panic!("Unexpected tag in update_parameter: \"{tag}\"")
                }
            },
            _ => {}
        }
    }
    fn update(&mut self) {
        //TODO
    }
}
