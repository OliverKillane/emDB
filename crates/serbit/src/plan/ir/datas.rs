use crate::plan::ir::{
    exprs::{BoolExpr, SizeExpr},
    keys,
    namer::{Assigned, Namer},
    primitives::Primitive,
    utils::Cases,
};

#[derive(Debug)]
pub enum DataType<'data_types, N: Namer> {
    Primitive(Primitive<N>),
    Struct {
        fields: Vec<Assigned<N, StructField<'data_types>>>,
    },
    Array {
        data_type: TypeCtxAssign<'data_types>,
        num_items: SizeExpr<TypeSizeCtx>,
    },
    Union(Cases<N, BoolExpr<TypeSizeCtx>, TypeCtxAssign<'data_types>>),
}

/// JUSTIFY: Cannot make path into a union or array.
///  - Paths are used to fetch previous values, for unions we cannot know which variant
///    is present, and implementing logic to determine if in the cirtumstance a value is
///    used, the union will be the correct variant is complex.
///  - for arrays, we would have paths dependent onf expressions - additional complexity
#[derive(Debug)]
pub enum DataTypePathNode {
    Primitive,
    Struct { field_index: usize },
}

#[derive(Debug)]
pub struct DataTypePath(Vec<DataTypePathNode>);

#[derive(Debug, Clone, Copy)]
pub struct TypeSizeCtx(usize);

#[derive(Debug, Clone, Copy)]
pub struct TypeBoolCtx(usize);

/// Maps [TypeCtx] from the outer context, to the inner context.
#[derive(Debug)]
pub struct TypeCtxAssign<'data_types> {
    pub key: keys::Datas<'data_types>,
    pub outer_to_inner: TypeCtxAssignMapping,
}

#[derive(Debug)]
pub struct TypeCtxAssignMapping {
    pub sizes: Vec<TypeSizeCtx>,
    pub bools: Vec<TypeBoolCtx>,
}

#[derive(Debug)]
pub struct StructField<'datas> {
    pub data: TypeCtxAssign<'datas>,
    pub visible: bool,
}