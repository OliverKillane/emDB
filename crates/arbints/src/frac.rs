// use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

// use crate::int::Int;

// pub trait Fractional:
//     Copy
//     + Clone
//     + Sized
//     + Add<Output = Self>
//     + Sub<Output = Self>
//     + Mul
//     + Div
//     + AddAssign
//     + SubAssign
//     + TryFrom<f32>
//     + TryFrom<f64>
// {
//     const WHOLE: usize;
//     const FRAC: usize;
//     const SIGNED: bool;

//     const MAX: Self;
//     const MIN: Self;

//     type Inner;
// }

// pub struct Frac<const WHOLE: usize, const FRAC: usize, const SIGNED: bool>(
//     Int<{WHOLE + FRAC}, SIGNED>,
// )
// where
//     Self: Fractional;
