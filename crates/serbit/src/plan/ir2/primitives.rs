use crate::plan::ir2::namer::Namer;
use std::{collections::HashMap, num::NonZero};

pub struct Bits(NonZero<u8>);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AnyInteger(isize);

#[derive(Debug)]
pub enum Endianesss {
    Big,
    Little,
}

#[derive(Debug)]
pub struct ByteType<N: Namer>{
    pub constraint: Constraint<u8, N>,
}

#[derive(Debug)]
pub struct IntegerType<N: Namer> {
    pub size: NonZero<u8>,
    pub endianness: Endianesss,
    pub constraint: Constraint<AnyInteger, N>,
}

#[derive(Debug)]
pub enum Primitive<N: Namer> {
    Bit,
    Byte(ByteType<N>),
    Integer(IntegerType<N>),
}

#[derive(Debug)]
pub enum PrimitiveValue<N: Namer> {
    Bit(bool),
    Byte(u8, ByteType<N>),
    Integer(AnyInteger, IntegerType<N>),
}

/// JUSTIFY: Using `T` rather than an [super::keys::Expr] (whch would allow for named constants)
///           - Keep simple, would need to consider incorrect expressions (invalid type)
#[derive(Debug)]
pub enum Constraint<T: Ord, N: Namer> {
    InclusiveRange { min: T, max: T },
    Variants { values: HashMap<N::Ident, T> },
}
