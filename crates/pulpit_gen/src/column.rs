use crate::{
    namer::Namer,
    table::{PushVec},
};

pub struct FieldState {
    pub datatype: Tokens<Type>,
    pub init: Tokens<ExprBlock>,
}

use quote::quote;
use quote_debug::Tokens;
use syn::{Type, ExprBlock, ExprClosure, ExprStruct, Ident, Item};

pub struct ColumnsConfig {
    pub primary_col: Column<PrimaryColumn>,
    pub assoc_columns: Vec<Column<AssocColumn>>,
}

impl ColumnsConfig {
    fn get_fields(&self) -> impl Iterator<Item = (&Ident, FieldIndex)> {
        self.primary_col
            .fields
            .mut_data
            .iter()
            .enumerate()
            .map(|t| (false, t))
            .chain(
                self.primary_col
                    .fields
                    .imm_data
                    .iter()
                    .enumerate()
                    .map(|t| (true, t)),
            )
            .map(|(imm, (field_num, field))| {
                (
                    &field.name,
                    FieldIndex::Primary(FieldIndexInner { imm, field_num }),
                )
            })
            .chain(self.assoc_columns.iter().enumerate().flat_map(
                |(ind, Column { col, fields })| {
                    fields
                        .mut_data
                        .iter()
                        .enumerate()
                        .map(|t| (false, t))
                        .chain(fields.imm_data.iter().enumerate().map(|t| (true, t)))
                        .map(move |(imm, (field_num, field))| {
                            (
                                &field.name,
                                FieldIndex::Assoc {
                                    ind,
                                    inner: FieldIndexInner { imm, field_num },
                                },
                            )
                        })
                },
            ))
    }
}

use std::collections::HashMap;

pub struct ColFieldsSelect {
    imm_fields: Vec<ColFieldInd>,
    mut_fields: Vec<ColFieldInd>,
}
pub struct ColsSelect {
    primary: Option<ColFieldsSelect>,
    assoc: HashMap<AssocInd, ColFieldsSelect>,
}

/// Combine multiple field indexes to get each field that needs to be accessed
/// from each column
pub fn combine_fields(fields: &[FieldIndex]) -> ColsSelect {
    let mut primary_select = ColFieldsSelect {
        imm_fields: Vec::new(),
        mut_fields: Vec::new(),
    };
    let mut assoc = HashMap::new();

    for field in fields {
        match field {
            FieldIndex::Primary(FieldIndexInner { imm, field_num }) => {
                if *imm {
                    &mut primary_select.imm_fields
                } else {
                    &mut primary_select.mut_fields
                }
                .push(*field_num);
            }
            FieldIndex::Assoc {
                ind,
                inner: FieldIndexInner { imm, field_num },
            } => {
                let cs = match assoc.get_mut(ind) {
                    Some(cs) => cs,
                    None => {
                        assoc.insert(
                            *ind,
                            ColFieldsSelect {
                                imm_fields: Vec::new(),
                                mut_fields: Vec::new(),
                            },
                        );
                        assoc
                            .get_mut(ind)
                            .expect("Already determined the associated column is not present")
                    }
                };
                if *imm {
                    &mut cs.imm_fields
                } else {
                    &mut cs.mut_fields
                }
                .push(*field_num);
            }
        }
    }
    ColsSelect {
        primary: if primary_select.imm_fields.is_empty() && primary_select.mut_fields.is_empty() {
            Some(primary_select)
        } else {
            None
        },
        assoc,
    }
}

pub struct FieldIndexInner {
    imm: bool,
    field_num: ColFieldInd,
}

pub type ColFieldInd = usize;
pub type AssocInd = usize;
pub enum FieldIndex {
    Primary(FieldIndexInner),
    Assoc {
        ind: AssocInd,
        inner: FieldIndexInner,
    },
}

#[enumtrait::quick_enum]
#[enumtrait::quick_from]
#[enumtrait::store(primary_column_enum)]
pub enum PrimaryColumn {
    PrimaryRetain,
}

#[enumtrait::quick_enum]
#[enumtrait::quick_from]
#[enumtrait::store(assoc_column_enum)]
pub enum AssocColumn {
    AssocBlocks,
}

pub struct Decoupling {
    /// Self::Imm(Get/Pull) -> (field1, field2, field3)
    conversion: Tokens<ExprClosure>,
    /// Types for each field in order
    types: Vec<Tokens<Type>>,
}

pub struct ColumnTypes {
    /// The type for the column
    pub concrete_type: Tokens<Type>,

    /// primaryWindow, AssocWindow, etc.
    pub kind_trait: Tokens<Type>,

    /// PrimaryWindowPull, PrimaryWindowAppend, AssocWindowPull, AssocWindow etc.
    pub access_trait: Tokens<Type>
}

#[enumtrait::store(column_gen_trait)]
pub trait ColumnGenerate {
    fn generate(&self, namer: &Namer, fields: &ColFields, prelude: &mut PushVec<Tokens<Item>>) -> ColumnTypes;

    fn decouple_imm(&self, imm_fields: &[Field], namer: &Namer) -> Decoupling;
    fn decouple_pull(&self, imm_fields: &[Field], namer: &Namer) -> Decoupling;
}

#[enumtrait::impl_trait(column_gen_trait for primary_column_enum)]
impl ColumnGenerate for PrimaryColumn {}

#[enumtrait::impl_trait(column_gen_trait for assoc_column_enum)]
impl ColumnGenerate for AssocColumn {}

pub struct Field {
    ty: Tokens<Type>,
    name: Ident,
}
pub struct FieldList {
    fields: Vec<(Ident, Tokens<Type>)>,
}

impl FieldList {
    fn tuple_type(&self) -> Tokens<Type> {
        let fields = self.fields.iter().map(|(_, ty)| ty);
        quote! { (#(#fields),*) }.into()
    }
}

pub struct ColFields {
    pub imm_data: Vec<Field>,
    pub mut_data: Vec<Field>,
}

pub struct Column<Col: ColumnGenerate> {
    pub col: Col,
    pub fields: ColFields,
}

pub struct PrimaryRetain {
    pub block_size: usize,
}

impl ColumnGenerate for PrimaryRetain {
    fn generate(&self, namer: &Namer, fields: &ColFields, prelude: &mut PushVec<Tokens<Item>>) -> ColumnTypes {
        let block_size = self.block_size;
        let imm_data = utils::tuple_type(&fields.imm_data);
        let mut_data = utils::tuple_type(&fields.mut_data);
        let lifetime = namer.window_lifetime();
        ColumnTypes {
            concrete_type: quote! ( PrimaryRetain<#imm_data,#mut_data,#block_size> ).into(),
            kind_trait: quote! ( PrimaryWindow<#lifetime, #imm_data,#mut_data> ).into(),
            access_trait: quote!( PrimaryWindowPull<#lifetime, #imm_data,#mut_data> ).into(),
        }
    }

    fn decouple_imm(&self, imm_fields: &[Field], namer: &Namer) -> Decoupling {
        utils::reference_decouple(imm_fields, namer)
    }

    fn decouple_pull(&self, imm_fields: &[Field], namer: &Namer) -> Decoupling {
        utils::reference_decouple(imm_fields, namer)
    }
}

pub struct AssocBlocks {
    pub block_size: usize,
}

impl ColumnGenerate for AssocBlocks {
    fn generate(&self, namer: &Namer, fields: &ColFields, prelude: &mut PushVec<Tokens<Item>>) -> ColumnTypes {
        let block_size = self.block_size;
        let imm_data = utils::tuple_type(&fields.imm_data);
        let mut_data = utils::tuple_type(&fields.mut_data);
        let lifetime = namer.window_lifetime();
        ColumnTypes {
            concrete_type: quote! ( AssocBlocks<#imm_data,#mut_data,#block_size> ).into(),
            kind_trait: quote! ( AssocWindow<#lifetime, #imm_data,#mut_data> ).into(),
            access_trait: quote!( AssocWindow<#lifetime, #imm_data,#mut_data> ).into(),
        }
    }

    fn decouple_imm(&self, imm_fields: &[Field], namer: &Namer) -> Decoupling {
        utils::reference_decouple(imm_fields, namer)
    }

    fn decouple_pull(&self, imm_fields: &[Field], namer: &Namer) -> Decoupling {
        utils::reference_decouple(imm_fields, namer)
    }
}

mod utils {
    use super::*;

    pub fn reference_decouple(imm_fields: &[Field], namer: &Namer) -> Decoupling {
        let imm_fields_type = tuple_type(imm_fields);
        let extract_fields: Vec<_> = (0..imm_fields.len()).map(|i| quote!(imm.#i)).collect();
        let lifetime = namer.window_lifetime();
        Decoupling {
            conversion: quote! {
                |imm: &imm_fields_type| {
                    (#(#extract_fields),*)
                }
            }
            .into(),
            types: imm_fields
                .iter()
                .map(|Field { ty, name }| quote!(&#lifetime #ty).into())
                .collect(),
        }
    }

    pub fn tuple_type(fields: &[Field]) -> Tokens<Type> {
        let fields = fields.iter().map(|Field { ty, name }| ty);
        quote! { (#(#fields),*) }.into()
    }
}
