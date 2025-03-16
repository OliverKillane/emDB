use better_arenas::prelude::*;
use std::marker::PhantomData;

pub mod expr;
pub mod plan;

/// [ExtraData] is used by analyses to add extra information.
///  - We can store a large struct of all applicable passes, containing the analyses types.
///  - As the types internals are only public to their analysis, we prevent any analyses
///    from overwriting each other's data.
pub struct Plan<Extra: ExtraData, Data: Arenas<Extra>> {
    pub stages: Data::Stages,
    pub items: Data::Items,
    pub ints: Data::Ints,
    pub bools: Data::Bools,
    _phantom: PhantomData<Data>,
}

pub trait ExtraData {
    type Item;
    type Stage;
    type Bool;
    type Int;
}

// JUSTIFY: Macros for types?
//          Aliases to make the [Arenas] trait easier to read.
//           - Rust requires we specify the associated type to eliminate any possible abiguities,
//             which makes the type signature huge.
macro_rules! bools_key {
    () => {
        <Self::Bools as Arena<store::Bool<Extra::Bool>>>::Key
    };
}
macro_rules! ints_key {
    () => {
        <Self::Ints as Arena<store::Int<Extra::Int, bools_key!()>>>::Key
    };
}
macro_rules! items_key {
    () => {
        <Self::Items as Arena<store::Item<Extra::Item, ints_key!(), bools_key!()>>>::Key
    };
}

/// A trait to gather all arena types concisely.
///  - means we do not need to specify arena types when taking a [plan], merely that we have some
///    type to choose them
///  - Means we can concisely add requirements atop the arena & extra type.
///
/// ### Jesus Christ, this is complicated.
/// yes. We are using associated types to create structs of types, and as functions on types (for the
/// [Store] trait).
///
/// The advantage is that we can now enforce at compile time:
///  - That we get the associated data we want for a given node (using [crate::utils::typeget::Has])
///  - That indices can always be read/written to (from [better_arenas]), so we do not have to [Option::unwrap]
///    every time we get an index we logically know must be present.
///  - That we get solid performance out of the box (bounds check elimination, associated data in-place)
pub trait Arenas<Extra: ExtraData> {
    type Bools: WriteArena<store::Bool<Extra::Bool>>;
    type Ints: WriteArena<store::Int<Extra::Int, bools_key!()>>;
    type Stages: WriteArena<store::Stage<Extra::Stage, items_key!(), ints_key!(), bools_key!()>>;
    type Items: WriteArena<store::Item<Extra::Item, ints_key!(), bools_key!()>>;
}

pub struct NodeData<Data, Extra> {
    pub data: Data,
    pub extra: Extra,
}

mod store {
    use super::*;

    macro_rules! make_store {
        ($name:ident for $module:ident, ($($id:ident),*)) => {
            pub struct $name<Extra, $($id),*>(PhantomData<(Extra, $($id),*)>);

            impl <Extra, $($id),*> Store for $name<Extra, $($id),*> {
                type Data<Key> = NodeData<$module::$name<Key, $($id),*>, Extra>;
            }
        }
    }

    make_store! {Stage for plan, (ItemKey, IntKey, BoolKey)}
    make_store! {Item for plan, (IntKey, BoolKey)}
    make_store! {Int for expr, (BoolKey)}
    make_store! {Bool for expr, ()}
}
