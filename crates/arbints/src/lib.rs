use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

pub type Bits = usize;
pub type Bytes = usize;

pub const fn bytes_from_bits(bits: Bits) -> Bytes {
    (bits + 7) / 8
}

pub mod frac;
pub mod int;
