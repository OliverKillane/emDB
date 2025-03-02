use std::num::NonZero;

use super::expr;

use enumtrait;

// Data Types
pub struct Bit;
pub struct Integer {
    pub signed: bool,
    // Only supporting up to 256bit integers
    pub bits: NonZero<u8>,
}

pub struct Byte;

#[enumtrait::quick_enum]
#[enumtrait::quick_from]
#[enumtrait::store(pub primitive_enum)]
pub enum Primitive {
    Byte,
    Bit,
    Integer,
}

pub struct Array {
    pub count: expr::Int,
    pub stage: Stage,
}

pub struct Case {
    pub condition: expr::Bool,
    pub data: Stage,
}

pub struct Choice {
    pub cases: Vec<Case>,
    pub otherwise: Stage,
}

#[enumtrait::quick_enum]
#[enumtrait::quick_from]
#[enumtrait::store(pub item_enum)]
pub enum Item {
    Array,
    Choice,
    Primitive,
}

pub struct Tuple<ItemIdx> {
    pub items: Vec<ItemIdx>, // SmallVec of indices
}

pub struct Until<StageIdx> {
    pub expr: expr::Bool,
    pub seq: Seq<StageIdx>,
}

pub struct Repeat {
    pub count: expr::Int,
    pub seq: Seq,
}

#[enumtrait::quick_enum]
#[enumtrait::quick_from]
#[enumtrait::store(pub item_stage)]
pub enum Stage<ItemIdx, StageIdx> {
    Tuple(Tuple<ItemIdx>),
    Repeat,
    Until(Until<StageIdx>),
}

pub struct Seq<StageIdx> {
    pub stages: Vec<StageIdx>,
}
