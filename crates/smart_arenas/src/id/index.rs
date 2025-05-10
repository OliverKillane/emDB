#![allow(non_camel_case_types)]

use std::{fmt::Debug, hash::Hash};

// JUSTIFY: [usize] not as the widest
//           - If using usize, might as well use a &T from an arena instead (ecepting mutability)
//           - u32 provides ~4 billion items, which is sufficient for most use cases
//           - Additionally on 32 bit targets, usize is 32 bits.
pub type WidestIndex = u32;

/// A trait for types used as raw offset/index into a [crate::alloc::AllocImpl]
///  - Additionally used for reference counts in [crate::arena::Share]
pub trait Index: Copy + Debug + Hash + Eq + Ord {
    const MAX: Self;
    const ZERO: Self;
    fn offset(self) -> WidestIndex;
    fn from_offset(offset: WidestIndex) -> Option<Self>;
    fn inc(&self) -> Self;
    fn dec(&self) -> Self;
}

macro_rules! impl_std_types {
    ($index_type:ty) => {
        impl Index for $index_type {
            const MAX: Self = <$index_type>::MAX;
            const ZERO: Self = 0;
            fn offset(self) -> WidestIndex {
                self as WidestIndex
            }
            fn from_offset(offset: WidestIndex) -> Option<Self> {
                if offset <= <$index_type>::MAX as WidestIndex {
                    Some(offset as $index_type)
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
impl_std_types!(u32);
