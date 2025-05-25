use crate::utils::{
    copy::{RawBits, new_raw_bits},
    size::{Bits, bytes},
};

pub trait Data: Copy + Sized {
    const SIZE: Bits;

    /// The raw bits (starting from offset 0) of the data
    fn from_bytes(data: RawBits<{ Self::SIZE }>) -> Self
    where
        [(); bytes(Self::SIZE)]:;

    /// Convert the data to bytes (starting at offset 0)
    fn to_bytes(self) -> RawBits<{ Self::SIZE }>
    where
        [(); bytes(Self::SIZE)]:;
}

/*
Be arbitrary
Le for u16, u32, u64, u128
Bit / bool
Byte
*/

impl Data for bool {
    const SIZE: Bits = 1;

    fn from_bytes(data: RawBits<{ Self::SIZE }>) -> Self
    where
        [(); bytes(Self::SIZE)]:,
    {
        data.inner()[0] & 1 != 0
    }

    fn to_bytes(self) -> RawBits<{ Self::SIZE }>
    where
        [(); bytes(Self::SIZE)]:,
    {
        new_raw_bits([if self { 0b1 } else { 0b0 }])
    }
}
