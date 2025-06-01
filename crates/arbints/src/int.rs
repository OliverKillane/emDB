use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};
use arbints_macros::generate_int;

pub struct TryFromError;

pub trait Integer:
    Copy
    + Clone
    + Sized
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + AddAssign
    + SubAssign
    + MulAssign
    + DivAssign
    + TryFrom<Self::Inner>
{
    const SIZE: usize;
    const SIGNED: bool;

    const MAX: Self;
    const MIN: Self;

    type Inner: Copy + From<Self>;

    fn extract(self) -> Self::Inner {
        self.into()
    }
}

pub struct Int<const SIZE: usize, const SIGNED: bool>(<Self as Integer>::Inner)
where
    Self: Integer;

generate_int!{}

impl<const SIZE: usize, const SIGNED: bool> Copy for Int<SIZE, SIGNED> where Self: Integer {}

impl<const SIZE: usize, const SIGNED: bool> Clone for Int<SIZE, SIGNED>
where
    Self: Integer,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<const SIZE: usize, const SIGNED: bool> Add for Int<SIZE, SIGNED>
where
    Self: Integer,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        todo!()
    }
}

impl<const SIZE: usize, const SIGNED: bool> Sub for Int<SIZE, SIGNED>
where
    Self: Integer,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        todo!()
    }
}

impl<const SIZE: usize, const SIGNED: bool> Mul for Int<SIZE, SIGNED>
where
    Self: Integer,
{
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        todo!()
    }
}

impl<const SIZE: usize, const SIGNED: bool> Div for Int<SIZE, SIGNED>
where
    Self: Integer,
{
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        todo!()
    }
}

impl<const SIZE: usize, const SIGNED: bool> AddAssign for Int<SIZE, SIGNED>
where
    Self: Integer,
{
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl<const SIZE: usize, const SIGNED: bool> SubAssign for Int<SIZE, SIGNED>
where
    Self: Integer,
{
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl<const SIZE: usize, const SIGNED: bool> MulAssign for Int<SIZE, SIGNED>
where
    Self: Integer,
{
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl<const SIZE: usize, const SIGNED: bool> DivAssign for Int<SIZE, SIGNED>
where
    Self: Integer,
{
    fn div_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}
