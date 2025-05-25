/// TODO: explain MSB first byte ordering
use super::{
    masks::byte_mask_upper,
    size::{Bits, bytes},
    truth::{Bool, Truth},
};

// TODO: document the byte ordering
pub struct RawBits<const SIZE: Bits>([u8; bytes(SIZE)])
where
    [(); bytes(SIZE)]:;

pub const fn new_raw_bits<const SIZE: Bits>(data: [u8; bytes(SIZE)]) -> RawBits<SIZE> {
    debug_assert!(
        data[0] >> (SIZE % 8) == 0,
        "Unused upper bits must be zeroed"
    );
    RawBits(data)
}

impl<const SIZE: Bits> RawBits<SIZE>
where
    [(); bytes(SIZE)]:,
{
    pub fn inner(self) -> [u8; bytes(SIZE)] {
        self.0
    }
}

/// Fast path if the data is byte aligned and byte sized
#[inline(always)]
pub const fn byte_aligned<const SIZE: Bits>(mut src: [u8; bytes(SIZE)]) -> RawBits<SIZE>
where
    Bool<{ SIZE % 8 == 0 }>: Truth,
    [(); bytes(SIZE)]:,
{
    src[0] &= !byte_mask_upper(SIZE % 8);
    new_raw_bits(src)
}

/// Case if the data ends at a byte boundary, we do not need to shift.
const fn use_larger_size(offset: Bits, size: Bits) -> bool {
    return bytes(size) < bytes(offset + size);
}

#[inline(always)]
pub unsafe fn same_bytes<const SIZE: Bits>(offset: Bits, src: [u8; bytes(SIZE)]) -> RawBits<SIZE>
where
    Bool<{ SIZE >= 1 }>: Truth,
    [(); bytes(SIZE)]:,
{
    debug_assert!(!use_larger_size(offset, SIZE));

    let left_shift = ((offset + SIZE) % 8) as u32;
    let right_shift = 8 - left_shift;

    let mut out = [0u8; bytes(SIZE)];

    for i in 1..bytes(SIZE) {
        out[i] = src[i].wrapping_shr(right_shift);
    }
    for i in 1..bytes(SIZE) {
        out[i] |= src[i - 1].wrapping_shl(left_shift);
    }
    out[0] = (src[0] & !byte_mask_upper(offset)).wrapping_shr(right_shift);
    RawBits(out)
}

/// For the case with data crossing byte boundaries.
#[inline(always)]
pub unsafe fn more_bytes<const SIZE: Bits>(
    offset: Bits,
    src: [u8; bytes(SIZE) + 1],
) -> RawBits<SIZE>
where
    Bool<{ SIZE >= 1 }>: Truth,
    Bool<{ bytes(SIZE) * 8 + 1 != SIZE }>: Truth,
    [(); bytes(SIZE)]:,
{
    debug_assert!(offset < 8, "bit shift must be less than 8");
    debug_assert!(use_larger_size(offset, SIZE));

    let left_shift = ((offset + SIZE) % 8) as u32;
    let right_shift = 8 - left_shift;
    let front_mask = !byte_mask_upper(SIZE % 8);

    let mut out = [0u8; bytes(SIZE)];

    for i in 0..bytes(SIZE) {
        out[i] = src[i + 1].wrapping_shr(right_shift);
    }
    for i in 0..bytes(SIZE) {
        out[i] |= src[i].wrapping_shl(left_shift);
    }
    out[0] &= front_mask;
    RawBits(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offset_more_bytes() {
        unsafe {
            assert_eq!(
                more_bytes::<4>(7, [0b00000001, 0b11100000]).inner(),
                [0b00001111]
            );
            assert_eq!(
                more_bytes::<8>(4, [0b00001111, 0b11110000]).inner(),
                [0b11111111]
            );
        }
    }

    mod more_size {
        use super::*;

        #[test]
        fn single_bytes() {
            unsafe {
                assert_eq!(
                    more_bytes::<4>(5, [0b00000111, 0b10000000]).inner(),
                    [0b00001111]
                );
                assert_eq!(
                    more_bytes::<8>(5, [0b00000111, 0b11111000]).inner(),
                    [0b11111111]
                );
                assert_eq!(
                    more_bytes::<8>(1, [0b01111111, 0b10000000]).inner(),
                    [0b11111111]
                );
            }
        }

        #[test]
        fn many_bytes() {
            unsafe {
                assert_eq!(
                    more_bytes::<18>(7, [0b00000001, 0b11111111, 0b11111111, 0b10000000]).inner(),
                    [0b00000011, 0b11111111, 0b11111111]
                );
                assert_eq!(
                    more_bytes::<18>(7, [0b00000001, 0b11110111, 0b11111110, 0b10000000]).inner(),
                    [0b00000011, 0b11101111, 0b11111101]
                )
            }
        }
    }

    mod same_size {
        use super::*;
        #[test]
        fn single_bytes() {
            unsafe {
                assert_eq!(same_bytes::<4>(3, [0b00011110]).inner(), [0b00001111]);
                assert_eq!(same_bytes::<1>(7, [0b00000001]).inner(), [0b00000001]);
                assert_eq!(same_bytes::<1>(0, [0b10000000]).inner(), [0b00000001]);
                assert_eq!(same_bytes::<2>(1, [0b01100000]).inner(), [0b00000011]);
            }
        }

        #[test]
        fn edge_values() {
            unsafe {
                assert_eq!(same_bytes::<1>(3, [0b00010000]).inner(), [0b00000001]);
                assert_eq!(same_bytes::<8>(0, [0b11111111]).inner(), [0b11111111]);
                assert_eq!(
                    same_bytes::<9>(0, [0b11111111, 0b10000000]).inner(),
                    [0b00000001, 0b11111111]
                );
                assert_eq!(
                    same_bytes::<17>(3, [0b00011111, 0b11111111, 0b11110000]).inner(),
                    [0b00000001, 0b11111111, 0b11111111]
                );
            }
        }
    }
}
