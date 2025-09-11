use std::sync::atomic::AtomicU32;

/// Atomic f32 which only supports loading and storing operations.
pub struct AtomicF32(AtomicU32);

impl AtomicF32 {
    pub fn new(val: f32) -> Self {
        AtomicF32(AtomicU32::new(val.to_bits()))
    }
    pub fn load(&self) -> f32 {
        f32::from_bits(self.0.load(std::sync::atomic::Ordering::Relaxed))
    }
    pub fn store(&self, val: f32) {
        self.0
            .store(val.to_bits(), std::sync::atomic::Ordering::Relaxed)
    }
}
