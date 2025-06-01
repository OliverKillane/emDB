use crate::utils::{size::Bits, truth::{Bool, Truth}};


trait Integer {
    const MAX: Self;
    const MIN: Self;
    const SIGNED: bool;
    type Underlying;
}

#[allow(non_camel_case_types)]
struct u1(<Self as Integer>::Underlying);

impl Integer for u1 {
    const MAX: Self = u1(1);
    const MIN: Self = u1(0);
    const SIGNED: bool = false;
    type Underlying = u8;
}
