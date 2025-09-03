use thiserror::Error;

#[derive(Error, Debug)]
pub enum WGPUError {
    #[error("No suitable GPU adapter found")]
    NoAdapter,

    #[error("No suitable Vulkan device found among {0} devices")]
    NoVulkanDevice(usize),

    #[error("Buffer size overflow: {0} elements × {1} bytes per element")]
    BufferSizeOverflow(usize, usize),

    #[error("Mapped memory size ({mapped}) is smaller than expected ({expected})")]
    InsufficientMappedMemory { mapped: u64, expected: u64 },

    #[error("Failed to find kernel module: {0}")]
    KernelNotFound(String),

    #[error("Failed to find compute queue family")]
    NoComputeQueue,

    #[error("wgpu error: {0}")]
    Wgpu(#[from] wgpu::Error),

    #[error("wgpu request device error: {0}")]
    WgpuRequestDevice(#[from] wgpu::RequestDeviceError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Other error: {0}")]
    Other(String),
}

impl From<Box<dyn std::error::Error>> for WGPUError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        WGPUError::Other(err.to_string())
    }
}
