use namer::{Assigned, Namer};
use primitives::PrimitiveValue;
use smart_arenas::prelude::*;

pub mod datas;
pub mod exprs;
pub mod namer;
pub mod primitives;
pub mod stages;

pub mod keys {
    use super::*;

    pub type Const<'id> = Key<'id, u32>;
    pub type Datas<'id> = Key<'id, u32>;
    pub type Stages<'id> = Key<'id, u32>;
}

pub mod arenas {
    use super::*;

    pub type Consts<'id, N: Namer> =
        Share<'id, keys::Const<'id>, Contig, u8, Assigned<N, PrimitiveValue<N>>>;
    pub type Datas<'id, N: Namer> =
        Share<'id, keys::Datas<'id>, Contig, u8, Assigned<N, datas::DataType<'id, N>>>;
    pub type Stages<'id, 'datas, N: Namer> =
        Own<'id, keys::Stages<'id>, Contig, Assigned<N, stages::Stage<'datas, 'id, N>>>;
}

pub struct Plan<'consts, 'datas, 'stages, N: Namer> {
    pub consts: arenas::Consts<'consts, N>,
    pub datas: arenas::Datas<'datas, N>,
    pub stages: arenas::Stages<'stages, 'datas, N>,
    pub messages: Vec<Assigned<N, keys::Stages<'stages>>>,
    pub namer: N,
}

pub mod utils {
    use super::*;

    #[derive(Debug)]
    pub struct Cases<N: Namer, E, C> {
        pub cases: Vec<Assigned<N, Case<E, C>>>,
        pub otherwise: Option<Assigned<N, C>>,
    }

    #[derive(Debug)]
    pub struct Case<E, C> {
        pub expr: E,
        pub case: C,
    }
}
