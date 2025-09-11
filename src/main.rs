use phase::simulation::ising::Ising;
use phase::simulation::with_egui;

fn main() {
    with_egui(Box::new(Ising::new()));
}
