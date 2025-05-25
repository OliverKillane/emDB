use super::size::Bits;

#[inline(always)]
pub const fn byte_mask_upper(upper: Bits) -> u8 {
    debug_assert!(upper <= 8);
    if upper == 0 {
        0
    } else {
        u8::MAX << (8 - upper)
    }
}

#[inline(always)]
pub const fn byte_mask_lower(lower: Bits) -> u8 {
    debug_assert!(lower <= 8);
    if lower == 0 { 0 } else { (1u8 << lower) - 1 }
}
