use std::num::NonZero;

use num_bigint::BigInt;
use smallvec::SmallVec;
use smart_arenas::prelude::*;

mod keys {
    use smart_arenas::prelude::Key;

    pub type Bool<'id> = Key<'id, u32>;
    pub type Int<'id> = Key<'id, u32>;
    pub type Item<'id> = Key<'id, u32>;
    pub type Stage<'id> = Key<'id, u16>;
    pub type Seq<'id> = Key<'id, u16>;
    pub type Msg<'id> = Key<'id, u8>;
}

pub struct Plan<'msgs, 'seqs, 'stages, 'items, 'bools, 'ints, N: Namer> {
    pub msgs: Share<'msgs, keys::Msg<'msgs>, Contig, u16, Assigned<N, Msg<'seqs>>>,
    pub seqs: Own<'seqs, keys::Seq<'seqs>, Contig, Seq<'stages>>,
    pub stages: Own<'stages, keys::Stage<'stages>, Contig, Stage<'items, 'seqs, 'bools>>,
    pub items:
        Share<'items, keys::Item<'items>, Contig, u16, Assigned<N, Item<'ints, 'items, 'bools>>>,
    pub bools: Own<'bools, keys::Bool<'bools>, Contig, Spanned<N, Bool<'bools, 'ints>>>,
    pub ints: Own<'ints, keys::Int<'ints>, Contig, Spanned<N, Int<'ints, 'bools, 'items>>>,
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

pub struct Integer {
    pub signed: bool,
    // Only supporting up to 256bit integers
    pub bits: NonZero<u8>,
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

// TODO: More primitives for ascii character
pub enum Primitive {
    Bit,
    Byte,
    Integer(Integer),
}

pub struct Case<'bools, To> {
    pub condition: keys::Bool<'bools>,
    pub to: To,
}

pub enum Item<'ints, 'items, 'bools> {
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
    Primitive(Primitive),
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

impl<'msgs, 'seqs, 'stages, 'items, 'bools, 'ints, N: Namer>
    Plan<'msgs, 'seqs, 'stages, 'items, 'bools, 'ints, N>
{
    pub fn new(
        token_msgs: Token<'msgs>,
        token_seqs: Token<'seqs>,
        token_stages: Token<'stages>,
        token_items: Token<'items>,
        token_bools: Token<'bools>,
        token_ints: Token<'ints>,
    ) -> Self {
        Self {
            msgs: Share::new(0, token_msgs),
            seqs: Own::new(0, token_seqs),
            stages: Own::new(0, token_stages),
            items: Share::new(0, token_items),
            bools: Own::new(0, token_bools),
            ints: Own::new(0, token_ints),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use smart_arenas::prelude::*;

    #[test]
    fn test_basic() {
        multiple_context!(
            token_msgs,
            token_seqs,
            token_stages,
            token_items,
            token_bools,
            token_ints => {
                // let plan = Plan::new(
                //     token_msgs,
                //     token_seqs,
                //     token_stages,
                //     token_items,
                //     token_bools,
                //     token_ints
                // );
            }
        );
    }
}
