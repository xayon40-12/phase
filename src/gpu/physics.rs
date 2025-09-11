use wgpu::{Buffer, Device, Queue};

pub mod ising;

/// Entries appearing in the Fragment shader corresponding to the [fragment_entry_point](FragmentInfo::fragment_entry_point) of [FragmentInfo].
#[derive(Clone)]
pub struct FragmentEntry<'a> {
    pub binding: u32,
    pub buffer: &'a Buffer,
    pub uniform: bool,
}

/// Fragment shader informations to be used by [RenderSquare](crate::simulation::render_square::RenderSquare) to performe the rendering of the [Physics] simulation.
pub struct FragmentInfo<'a> {
    pub fragment_entry_point: &'a str,
    pub entries: Vec<FragmentEntry<'a>>,
}

/// Physics trait to define the minimum requierement for a physics simulation to be able to compute and render in the GPU with [RenderSquare](crate::simulation::render_square::RenderSquare).
pub trait Physics: Send + Sync + 'static {
    /// Update the physics, which would principally be a compute pipeline.
    fn update(&mut self, device: &Device, queue: &Queue);
    /// Necessary fragment buffer informations for the [RenderSquare](crate::simulation::render_square::RenderSquare).
    fn wgpu_fragment_info(&self) -> FragmentInfo;
}
