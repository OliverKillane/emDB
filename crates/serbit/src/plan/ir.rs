use std::{hash::Hash, marker::PhantomData, num::NonZero};

use indexmap::IndexSet;
use num_bigint::BigInt;
use quote_debug::Tokens;
use smallvec::SmallVec;
use smart_arenas::prelude::*;

use crate::utils::options::Options;

pub mod keys {
    use smart_arenas::prelude::Key;

    pub type Bool<'id> = Key<'id, u32>;
    pub type Int<'id> = Key<'id, u32>;
    pub type Item<'id> = Key<'id, u32>;
    pub type Const<'id> = Key<'id, u16>;
    pub type Stage<'id> = Key<'id, u16>;
    pub type Seq<'id> = Key<'id, u16>;
    pub type Msg<'id> = Key<'id, u8>;

    #[derive(PartialEq, Eq, Hash)]
    pub enum Object<'msgs, 'seqs, 'stages, 'items, 'ints, 'bools, 'consts> {
        Msg(Msg<'msgs>),
        Seq(Seq<'seqs>),
        Stage(Stage<'stages>),
        Item(Item<'items>),
        Int(Int<'ints>),
        Bool(Bool<'bools>),
        Const(Const<'consts>),
    }
}


/*
Differentiate between context, and type

item -> owned
type -> shared + has a ctx argument set

Type just has Ctx {
    usages of ctx
}

Type {
    Contains Ctx in expressions
    -> use
}

Message, Seq contains 

For a type
type contains array with size

For a stage
if, while conditions depend on previous

dependencies
 - in a type - only ctx
 - in a stage, we need the item identifier + the internal identifier

type includes nullvalue / optional or value

type (ctx) {
    x: thingy(ctx.foo),
    y: thingy(ctx.bar)
}

For a stage:
 - item is a function of item(ctx0: outer_ctx23) -> (itemctx)
 - each item adds to context, context grows
 - type has typepath, which expresses the choice / traversal of the path
 - each stage gives info

Primitive()
Tuple(index)
Array(index)
Choice(
    if (
        itemctx: Blagh
    )
)

Traverse: check set of available items
Traverse: check the items paths are valid

Item is a ctx in, all out

msgs, 
stages, 
items, A special usage of a stage
types, take in ctx, make use of ctx

type {
    tuple: (
        Usage(ctxbind) => type
    )
    Reference does ctxbind
}
CtxBool
Expr
*/

pub struct Plan<'msgs, 'stages, 'items, 'bools, 'ints, 'consts, N: Namer> {
    pub msgs: Share<'msgs, keys::Msg<'msgs>, Contig, u16, Assigned<N, Msg<'stages>>>,
    pub stages: Own<'stages, keys::Stage<'stages>, Contig, Stage<'items, 'stages, 'bools>>,
    pub items: App<'items, keys::Item<'items>, Contig, Assigned<N, Item<'ints, 'items, N>>>,
    pub bools: App<'bools, keys::Bool<'bools>, Contig, Spanned<N, Bool<'bools, 'ints>>>,
    pub ints: App<'ints, keys::Int<'ints>, Contig, Spanned<N, Int<'ints, 'bools, 'items>>>,
    pub consts: Share<'consts, keys::Const<'consts>, Contig, u16, Assigned<N, Constant>>,
    pub namer: N,
}

pub trait Namer: std::fmt::Debug + Eq {
    /// A span, representing a location in the source
    type Span: From<Self::Ident> + Clone + std::fmt::Debug;

    /// A raw name (no span)
    type Name: Eq + Hash + From<Self::Ident> + Clone + std::fmt::Debug;

    /// An identifier, which includes a span
    type Ident: Eq + Hash + Clone;
}

// TODO: Documentation generation
pub struct Doced<'msgs, 'stages, 'items, 'bools, 'ints, 'consts, N: Namer, Data> {
    pub generate: Option<
        Box<dyn Fn(&Plan<'msgs, 'stages, 'items, 'bools, 'ints, 'consts, N>) -> String>,
    >,
    pub data: Data,
}

pub struct Assigned<N: Namer, Data> {
    pub ident: N::Ident,
    pub data: Spanned<N, Data>,
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

pub enum Item<'ints, 'items, N: Namer> {
    Array {
        count: keys::Int<'ints>,
        item: keys::Item<'items>,
    },
    Tuple {
        items: SmallVec<[keys::Item<'items>; 11]>,
    },
    Primitive(Primitive<ConstraintAssoc<N>>),
}

pub enum Stage<'items, 'stages, 'bools> {
    Single(keys::Item<'items>),
    Repeat {
        count: keys::Item<'items>,
        stage: keys::Stage<'stages>,
    },
    Choice {
        cases: Options<Case<'bools, keys::Stage<'stages>>>,
        otherwise: Option<keys::Item<'items>>,
    },
    Until {
        expr: keys::Bool<'bools>,
        stage: keys::Stage<'stages>,
        include_end: bool,
    },
    Series {
        stages: SmallVec<[keys::Stage<'stages>; 8]>
    }
}

pub struct Msg<'stages> {
    pub stage: keys::Stage<'stages>,
}
