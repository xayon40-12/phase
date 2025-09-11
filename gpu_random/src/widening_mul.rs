/// Widening multiplication for u32 without the use of u64 which is necessary for WebGPU as it does not support u64.
#[inline(always)]
pub fn widening_mul_u32(a: u32, b: u32) -> (u32, u32) {
    let a0 = a & 0xFFFF;
    let a1 = a >> 16;
    let b0 = b & 0xFFFF;
    let b1 = b >> 16;

    let p0 = a0 * b0;
    let p1 = a0 * b1;
    let p2 = a1 * b0;
    let p3 = a1 * b1;

    let lo_low16 = p0 & 0xFFFF;
    let carry_mid = (p0 >> 16) + (p1 & 0xFFFF) + (p2 & 0xFFFF);
    let lo_high16 = carry_mid & 0xFFFF;
    let carry_hi = carry_mid >> 16;

    let lo = (lo_high16 << 16) | lo_low16;
    let hi = (p1 >> 16) + (p2 >> 16) + p3 + carry_hi;
    (lo, hi)
}

/// Test that [widening_mul_u32] reproduces a multiplication of two u32 casted as u64.
#[test]
fn test_widening() {
    use crate::{GPURng, philox::Philox4x32};
    let mut phi = Philox4x32::new(0, 0);
    for _ in 0..100000 {
        let a = phi.next_u32();
        let b = phi.next_u32();
        let m = a as u64 * b as u64;
        let (lo, hi) = widening_mul_u32(a, b);
        assert!(m as u32 == lo);
        assert!((m >> 32) as u32 == hi);
    }
}
