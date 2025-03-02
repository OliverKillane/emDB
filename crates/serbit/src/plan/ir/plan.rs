use std::num::NonZero;

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

pub struct Array<ItemKey, IntKey> {
    pub count: IntKey,
    pub stage: ItemKey,
}

pub struct Case<ItemKey, BoolKey> {
    pub condition: BoolKey,
    pub data: ItemKey,
}

pub struct Choice<ItemKey, BoolKey> {
    pub cases: Vec<Case<ItemKey, BoolKey>>,
    pub otherwise: ItemKey,
}

pub struct Tuple<ItemKey> {
    pub items: Vec<ItemKey>, // SmallVec of indices
}

#[enumtrait::quick_enum]
#[enumtrait::quick_from]
#[enumtrait::store(pub item_enum)]
pub enum Item<ItemKey, IntKey, BoolKey> {
    Array(Array<ItemKey, IntKey>),
    Choice(Choice<ItemKey, BoolKey>),
    Tuple(Tuple<ItemKey>),
    Primitive,
}

pub struct Until<StageKey, BoolKey> {
    pub expr: BoolKey,
    pub seq: StageKey,
}

pub struct Repeat<StageKey, IntKey> {
    pub count: IntKey,
    pub seq: StageKey,
}

pub struct Seq<StageKey> {
    pub stages: Vec<StageKey>,
}

#[enumtrait::quick_from]
#[enumtrait::store(pub item_stage)]
pub enum Stage<ItemKey, StageKey, IntKey, BoolKey> {
    Item(Item<ItemKey, IntKey, BoolKey>),
    Repeat(Repeat<StageKey, IntKey>),
    Until(Until<StageKey, BoolKey>),
    Seq(Seq<StageKey>),
}
