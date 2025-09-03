use phase::simulation::ising::Ising;
use phase::simulation::with_egui;

fn main() {
    with_egui(Box::new(|| {
        Box::new(Ising::new(kernel::IsingCtx {
            width: 1024,
            height: 1024,
            temperature: 1.0,
            chemical_potential: 1.0,
        }))
    }));
}
