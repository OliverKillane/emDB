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
    pub type Stages<'id, 'data_types, N: Namer> =
        Own<'id, keys::Stages<'id>, Contig, Assigned<N, stages::Stage<'data_types, 'id, N>>>;
}

pub struct Plan<'consts, 'data_types, 'msg_stages, N: namer::Namer> {
    pub consts: arenas::Consts<'consts, N>,
    pub datas: arenas::Datas<'data_types, N>,
    pub stages: arenas::Stages<'msg_stages, 'data_types, N>,
    pub namer: N,
}

pub mod utils {
    use crate::plan::ir2::namer::{Assigned, Namer};

    #[derive(Debug)]
    pub struct Cases<N: Namer, E, C> {
        pub cases: Vec<Case<N, E, C>>,
        pub otherwise: Option<Assigned<N, C>>,
    }

    #[derive(Debug)]
    pub struct Case<N: Namer, E, C> {
        pub ident: N::Ident,
        pub expr: E,
        pub case: C,
    }
}
