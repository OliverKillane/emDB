use num_bigint::BigInt;
use smallvec::SmallVec;
use smart_arenas::{alloc, arena};
use std::{num::NonZero, ops::RangeInclusive};

pub struct Plan<N: Naming> {
    pub msgs: arena::Share<keys::Msg, alloc::Contig, u16, Assigned<N, Msg>>,
    pub seqs: arena::Own<keys::Seq, alloc::Contig, Seq>,
    pub stages: arena::Own<keys::Stage, alloc::Contig, Stage>,
    pub items: arena::Share<keys::Item, alloc::Contig, u16, Assigned<N, Item>>,
    pub bools: arena::Own<keys::Bool, alloc::Contig, Spanned<N, Bool>>,
    pub ints: arena::Own<keys::Int, alloc::Contig, Spanned<N, Int>>,
}

pub mod keys {
    use smart_arenas::define_keys;
    define_keys! {
        Item => u16,
        Stage => u16,
        Bool => u16,
        Int => u16,
        Seq => u16,
        Msg => u16,
    }
}

pub trait Naming {
    type Span;
    type Ident;
}

pub struct Assigned<N: Naming, Data> {
    pub name: N::Ident,
    pub spanned_data: Spanned<N, Data>,
}

pub struct Spanned<N: Naming, Data> {
    pub span: N::Span,
    pub data: Data,
}

pub enum MathBinOp {
    Subtract,
    Multiply,
    Add,
}

pub enum Int {
    Const { value: BigInt, kind: Integer }, // TODO(oliverkillane): BigInt heap allocates (vector, switch to smallvec?)
    Bin(MathBinOp, keys::Int, keys::Int),
    Ref(keys::Item),
    Choice(keys::Bool, keys::Int, keys::Int),
}

pub enum LogicalBinOp {
    And,
    Or,
}

pub enum ArithBinOp {
    Eq,
    Gt,
}

pub enum Bool {
    Const(bool),
    Not(keys::Bool),
    Logic(LogicalBinOp, keys::Bool, keys::Bool),
    Arith(ArithBinOp, keys::Int, keys::Int),
}

pub struct Bit;
pub struct Integer {
    pub signed: bool,
    // Only supporting up to 256bit integers
    pub bits: NonZero<u8>,
}

pub struct Byte;

#[enumtrait::quick_enum]
#[enumtrait::quick_from]
#[enumtrait::store(pub item_primitive)]
pub enum Primitive {
    Byte,
    Bit,
    Integer,
}

pub struct Array {
    pub count: keys::Int,
    pub item: keys::Item,
}

pub struct Case {
    pub condition: keys::Bool,
    pub data: keys::Item,
}

pub struct Choice {
    pub cases: SmallVec<[Case; 2]>,
    pub otherwise: keys::Item,
}

pub struct Tuple {
    pub items: SmallVec<[keys::Item; 11]>,
}

#[enumtrait::quick_enum]
#[enumtrait::quick_from]
#[enumtrait::store(pub item_enum)]
pub enum Item {
    Array,
    Choice,
    Tuple,
    Primitive,
}

pub struct Until {
    pub expr: keys::Bool,
    pub seq: keys::Seq,
}

pub struct Repeat {
    pub count: keys::Item,
    pub seq: keys::Seq,
}

pub struct Single {
    pub item: keys::Item,
}

#[enumtrait::quick_enum]
#[enumtrait::quick_from]
#[enumtrait::store(pub item_stage)]
pub enum Stage {
    Single,
    Repeat,
    Until,
}

pub struct Seq {
    pub stages: SmallVec<[keys::Stage; 8]>,
}

pub struct Msg {
    pub seq: keys::Seq,
}

impl Integer {
    pub fn range(&self) -> RangeInclusive<BigInt> {
        let bits = self.bits.get();
        if self.signed {
            // min = -2^(bits-1), max = 2^(bits-1) - 1
            let half = bits - 1;
            let max = (BigInt::from(1) << half) - 1u32;
            let min = -(&max + 1u32);
            min..=max
        } else {
            // min = 0, max = 2^bits - 1
            let max = (BigInt::from(1) << bits) - 1u32;
            BigInt::ZERO..=max
        }
    }
}
