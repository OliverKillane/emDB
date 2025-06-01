use std::{marker::PhantomData, num::NonZero};

use indexmap::IndexSet;
use num_bigint::BigInt;
use smallvec::SmallVec;
use smart_arenas::prelude::*;

mod keys {
    use smart_arenas::prelude::Key;

    pub type Bool<'id> = Key<'id, u32>;
    pub type Int<'id> = Key<'id, u32>;
    pub type Item<'id> = Key<'id, u32>;
    pub type Const<'id> = Key<'id, u16>;
    pub type Stage<'id> = Key<'id, u16>;
    pub type Seq<'id> = Key<'id, u16>;
    pub type Msg<'id> = Key<'id, u8>;
}

pub struct Plan<'msgs, 'seqs, 'stages, 'items, 'bools, 'ints, 'consts, N: Namer> {
    pub msgs: Share<'msgs, keys::Msg<'msgs>, Contig, u16, Assigned<N, Msg<'seqs>>>,
    pub seqs: Own<'seqs, keys::Seq<'seqs>, Contig, Seq<'stages>>,
    pub stages: Own<'stages, keys::Stage<'stages>, Contig, Stage<'items, 'seqs, 'bools>>,
    pub items:
        Share<'items, keys::Item<'items>, Contig, u16, Assigned<N, Item<'ints, 'items, 'bools, N>>>,
    pub bools: Own<'bools, keys::Bool<'bools>, Contig, Spanned<N, Bool<'bools, 'ints>>>,
    pub ints: Own<'ints, keys::Int<'ints>, Contig, Spanned<N, Int<'ints, 'bools, 'items>>>,
    pub consts: Share<'consts, keys::Const<'consts>, Contig, u16, Assigned<N, Constant>>,
    pub namer: N,
}

pub trait Namer {
    type Span;
    type Ident;
}

pub struct Assigned<N: Namer, Data> {
    pub name: N::Ident,
    pub spanned_data: Spanned<N, Data>,
}

pub struct Spanned<N: Namer, Data> {
    pub span: N::Span,
    pub data: Data,
}

pub enum Encoding {
    BigEndian,
    LittleEndian,
}

pub struct Integer {
    pub signed: bool,
    // Only supporting up to 256bit integers
    pub bits: NonZero<u8>,
    pub encoding: Encoding,
}

pub enum MathBinOp {
    Subtract,
    Multiply,
    Add,
}

pub enum Int<'ints, 'bools, 'items> {
    Const { value: BigInt, kind: Integer }, // TODO(oliverkillane): BigInt heap allocates (vector, switch to smallvec?)
    Bin(MathBinOp, keys::Int<'ints>, keys::Int<'ints>),
    Ref(keys::Item<'items>),
    Choice(keys::Bool<'bools>, keys::Int<'ints>, keys::Int<'ints>),
}

pub enum LogicalBinOp {
    And,
    Or,
}

pub enum ArithBinOp {
    Eq,
    Gt,
}

pub enum Bool<'bools, 'ints> {
    Const(bool),
    Not(keys::Bool<'bools>),
    Logic(LogicalBinOp, keys::Bool<'bools>, keys::Bool<'bools>),
    Arith(ArithBinOp, keys::Int<'ints>, keys::Int<'ints>),
}

pub trait PrimitiveAssoc {
    type Bit;
    type Byte;
    type Int;
}

pub enum Primitive<P: PrimitiveAssoc> {
    Bit(P::Bit),
    Byte(P::Byte),
    Integer { underlying: Integer, assoc: P::Int },
}

pub struct Value<Value, N: Namer> {
    pub name: N::Ident,
    pub value: Value,
}

pub enum Constraint<V, N: Namer> {
    Set { values: IndexSet<Value<V, N>> },
    Range { min: V, max: V },
    None,
}

pub struct ConstraintAssoc<N: Namer>(PhantomData<N>);

impl<N: Namer> PrimitiveAssoc for ConstraintAssoc<N> {
    type Bit = (); // TODO(oliverkillane): Maybe we should associate some data?
    type Byte = Constraint<u8, N>;
    type Int = Constraint<isize, N>;
}

pub struct ConstAssoc;

impl PrimitiveAssoc for ConstAssoc {
    type Bit = bool;
    type Byte = u8;
    type Int = isize;
}

pub struct Case<'bools, To> {
    pub condition: keys::Bool<'bools>,
    pub to: To,
}

pub struct Constant {
    pub value: Primitive<ConstAssoc>,
}

pub enum Item<'ints, 'items, 'bools, N: Namer> {
    Array {
        count: keys::Int<'ints>,
        item: keys::Item<'items>,
    },
    /// SEM: All cases must be the same size
    Union {
        cases: SmallVec<[Case<'bools, keys::Item<'items>>; 2]>,
        otherwise: keys::Item<'items>,
    },
    Tuple {
        items: SmallVec<[keys::Item<'items>; 11]>,
    },
    Primitive(Primitive<ConstraintAssoc<N>>),
}

pub enum Stage<'items, 'seqs, 'bools> {
    Single(keys::Item<'items>),
    Repeat {
        count: keys::Item<'items>,
        seq: keys::Seq<'seqs>,
    },
    Choice {
        cases: SmallVec<[Case<'bools, keys::Seq<'seqs>>; 2]>,
        otherwise: keys::Item<'items>,
    },
    Until {
        expr: keys::Bool<'bools>,
        seq: keys::Seq<'seqs>,
        include_end: bool,
    },
}

pub struct Seq<'stages> {
    pub stages: SmallVec<[keys::Stage<'stages>; 8]>,
}

pub struct Msg<'seqs> {
    pub seq: keys::Seq<'seqs>,
}

impl<'msgs, 'seqs, 'stages, 'items, 'bools, 'ints, 'consts, N: Namer>
    Plan<'msgs, 'seqs, 'stages, 'items, 'bools, 'ints, 'consts, N>
{
    pub fn new(
        token_msgs: Token<'msgs>,
        token_seqs: Token<'seqs>,
        token_stages: Token<'stages>,
        token_items: Token<'items>,
        token_bools: Token<'bools>,
        token_ints: Token<'ints>,
        token_consts: Token<'consts>,
        namer: N,
    ) -> Self {
        Self {
            msgs: Share::new(0, token_msgs),
            seqs: Own::new(0, token_seqs),
            stages: Own::new(0, token_stages),
            items: Share::new(0, token_items),
            bools: Own::new(0, token_bools),
            ints: Own::new(0, token_ints),
            consts: Share::new(0, token_consts),
            namer,
        }
    }
}

#[cfg(test)]
mod tests {
    struct TestNamer;

    impl Namer for TestNamer {
        type Span = usize;
        type Ident = &'static str;
    }

    use super::*;

    #[test]
    fn test_basic() {
        multiple_context!(
            token_msgs,
            token_seqs,
            token_stages,
            token_items,
            token_bools,
            token_ints,
            token_consts => {
                let mut plan = Plan::new(
                    token_msgs, token_seqs, token_stages, token_items, token_bools, token_ints, token_consts, TestNamer);

                plan.consts.insert(
                    Assigned { name: "foo", spanned_data: Spanned { span: 0, data: Constant { value: Primitive::Bit(true) } }, }
                );
            }
        );
    }
}
