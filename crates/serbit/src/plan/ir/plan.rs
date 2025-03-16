use smallvec::SmallVec;
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
    pub item: ItemKey,
}

pub struct Case<ItemKey, BoolKey> {
    pub condition: BoolKey,
    pub data: ItemKey,
}

pub struct Choice<ItemKey, BoolKey> {
    pub cases: SmallVec<[Case<ItemKey, BoolKey>; 2]>,
    pub otherwise: ItemKey,
}

pub struct Tuple<ItemKey> {
    pub items: SmallVec<[ItemKey; 11]>,
}

// JUSTIFY: Using 32 bytes as a maximum size (4 * u64)
//           - Want to keep nodes small, part of an enum, so only max size matters
//           - Increasing the in-place capacity of the smallvecs, until hitting this limit
const _: () = assert!(std::mem::size_of::<Tuple<u16>>() == 32);
const _: () = assert!(std::mem::size_of::<Choice<u16, u16>>() == 32);
const _: () = assert!(std::mem::size_of::<Array<u16, u16>>() == 4);
const _: () = assert!(std::mem::size_of::<Primitive>() == 2);

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
    pub stages: SmallVec<[StageKey; 8]>,
}

// TODO: Determine more optimal size, versus heap allocation tradeoff
const _: () = assert!(std::mem::size_of::<Repeat<u16, u16>>() == 4);
const _: () = assert!(std::mem::size_of::<Until<u16, u16>>() == 4);
const _: () = assert!(std::mem::size_of::<Seq<u16>>() == 32);

#[enumtrait::store(pub item_stage)]
pub enum Stage<StageKey, ItemKey, IntKey, BoolKey> {
    Item(ItemKey),
    Repeat(Repeat<StageKey, IntKey>),
    Until(Until<StageKey, BoolKey>),
    Seq(Seq<StageKey>),
}
