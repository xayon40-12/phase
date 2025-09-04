use wgpu::{Buffer, Device, Queue};

pub mod ising;

type WGPUInfo<'a> = (&'a str, Vec<(u32, &'a Buffer, bool)>);

pub trait Physics: Send + Sync + 'static {
    fn update(&mut self, device: &Device, queue: &Queue);
    fn wgpu_info(&self) -> WGPUInfo;
}
