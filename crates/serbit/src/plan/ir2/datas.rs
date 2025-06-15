use crate::plan::ir2::{
    exprs::{BoolExpr, IntExpr},
    keys,
    namer::{Assigned, Namer},
    primitives::Primitive,
    utils::Cases,
};

#[derive(Debug)]
pub enum DataType<'data_types, N: Namer> {
    Primitive(Primitive<N>),
    Struct {
        fields: Vec<Assigned<N, TypeCtxAssign<'data_types>>>,
    },
    Array {
        data_type: TypeCtxAssign<'data_types>,
        num_items: IntExpr<TypeCtx>,
    },
    Union(Cases<N, BoolExpr<TypeCtx>, TypeCtxAssign<'data_types>>),
}

/// JUSTIFY: Cannot make path into a union.
///  - Paths are used to fetch previous values, for unions we cannot know which variant
///    is present, and implementing logic to determine if in the cirtumstance a value is
///    used, the union will be the correct variant is complex.
#[derive(Debug)]
pub enum DataTypePathNode {
    Primitive,
    Struct { field_index: usize },
    Array,
}

#[derive(Debug)]
pub struct DataTypePath(Vec<DataTypePathNode>);

#[derive(Debug, Clone, Copy)]
pub struct TypeCtx(usize);

/// Maps [TypeCtx] from the outer context, to the inner context.
#[derive(Debug)]
pub struct TypeCtxAssign<'data_types> {
    pub key: keys::Datas<'data_types>,
    pub outer_to_inner: TypeCtxMap,
}

#[derive(Debug)]
pub struct TypeCtxMap {
    pub outer_to_inner: Vec<TypeCtx>,
}

impl TypeCtxMap {
    fn translate(&self, ctx: &TypeCtx) -> TypeCtx {
        *self.outer_to_inner.get(ctx.0).unwrap()
    }
}
