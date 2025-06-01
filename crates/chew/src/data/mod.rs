use crate::utils::{
    copy::{RawBits, new_raw_bits},
    size::{Bits, bytes},
};

pub mod int;

pub trait Data: Copy + Sized {
    const SIZE: Bits;

    fn deserialize(data: RawBits<{ Self::SIZE }>) -> Self
    where
        [(); bytes(Self::SIZE)]:;

    fn serialize(self) -> RawBits<{ Self::SIZE }>
    where
        [(); bytes(Self::SIZE)]:;
}

/*
*/

impl Data for bool {
    const SIZE: Bits = 1;

    fn deserialize(data: RawBits<{ Self::SIZE }>) -> Self
    where
        [(); bytes(Self::SIZE)]:,
    {
        data.inner()[0] & 1 != 0
    }

    fn serialize(self) -> RawBits<{ Self::SIZE }>
    where
        [(); bytes(Self::SIZE)]:,
    {
        new_raw_bits([if self { 0b1 } else { 0b0 }])
    }
}
