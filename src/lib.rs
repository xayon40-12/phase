pub mod error;
pub mod gpu;
pub mod simulation;

pub const SPIRV: &[u8] = include_bytes!(env!("KERNEL_SPV_PATH"));
