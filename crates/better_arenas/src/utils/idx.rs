// JUSTIFY: For consistency in the unsigned type names
#![allow(non_camel_case_types)]

use std::{fmt::Debug, hash::Hash};

pub trait IdxInt: Copy + Debug + Hash + Eq  {
    const MAX: Self;
    const ZERO: Self;
    fn offset(self) -> usize;
    fn from_offset(offset: usize) -> Option<Self>;
    fn inc(&self) -> Self;
    fn dec(&self) -> Self;
}

macro_rules! impl_std_types {
    ($t:ty) => {
        impl IdxInt for $t {
            const MAX: Self = <$t>::MAX;
            const ZERO: Self = 0;
            fn offset(self) -> usize {
                self as usize
            }
            fn from_offset(offset: usize) -> Option<Self> {
                if offset <= <$t>::MAX as usize {
                    Some(offset as $t)
                } else {
                    None
                }
            }
            fn inc(&self) -> Self {
                self + 1
            }
            fn dec(&self) -> Self {
                self - 1
            }
        }
    };
}

impl_std_types!(u8);
impl_std_types!(u16);

#[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
impl_std_types!(u32);

#[cfg(target_pointer_width = "64")]
impl_std_types!(u64);
