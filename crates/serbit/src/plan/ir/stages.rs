use std::collections::HashMap;

use crate::plan::ir::{
    datas::{DataTypePath, TypeSizeCtx},
    exprs::{BoolExpr, SizeExpr},
    keys,
    namer::Namer,
    utils::Cases,
};

pub struct StageCtx(usize);

pub enum Stage<'data_types, 'msg_stages, N: Namer> {
    Instance {
        input_ctx: HashMap<StageCtx, TypeSizeCtx>,
        data: keys::Datas<'data_types>,
        append_to_msg_ctx: HashMap<StageCtx, DataTypePath>,
    },
    Sequence {
        stages: Vec<keys::Stages<'msg_stages>>,
    },
    Repeat {
        size: SizeExpr<StageCtx>,
        stage: keys::Stages<'msg_stages>,
    },
    Until {
        condition: BoolExpr<StageCtx>,
        stage: keys::Stages<'msg_stages>,
        include_end: bool,
    },
    Choice(Cases<N, BoolExpr<StageCtx>, keys::Stages<'msg_stages>>),
}
