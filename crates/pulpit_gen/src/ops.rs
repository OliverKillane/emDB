use crate::index::IndexGen;
use proc_macro2::TokenStream;

use super::{FieldId, FieldIndex, Group, OpName, Table};
use quote::{quote, ToTokens};
#[enumtrait::quick_enum]
#[enumtrait::store(ops_kind_enum)]
pub enum Kind {
    Insert,
    Update,
    Get,
    Brw,
    BrwMut,
    Count,
}

pub struct OpRes {
    ///  Struct definitions and aliases for use outside of the generated method
    pub prelude: TokenStream,

    /// The method.
    pub method: TokenStream,
}

#[enumtrait::store(ops_gen_trait)]
pub trait OpGen {
    fn generate(
        &self,
        table: &Table,
        name: &OpName,
        tks_before: impl ToTokens,
        tks_after: impl ToTokens,
    ) -> OpRes;
}

#[enumtrait::impl_trait(ops_gen_trait for ops_kind_enum)]
impl OpGen for Kind {}

pub struct Insert;

impl OpGen for Insert {
    fn generate(
        &self,
        table: &Table,
        name: &OpName,
        tks_before: impl ToTokens,
        tks_after: impl ToTokens,
    ) -> OpRes {
        let mut field_params = Vec::with_capacity(table.fields.len());
        let mut group_assigns = Vec::with_capacity(table.groups.len());
        for (
            i,
            Group {
                col,
                mut_fields,
                imm_fields,
            },
        ) in table.groups.iter().enumerate()
        {
            let mut imm_assign = Vec::with_capacity(imm_fields.len());
            for (f, imm_field_t) in imm_fields.iter().enumerate() {
                let id = table
                    .fields
                    .get_by_right(&FieldIndex {
                        group: i,
                        mutable: false,
                        place: f,
                    })
                    .unwrap();
                field_params.push(quote! {#id: #imm_field_t});
                imm_assign.push(id);
            }
            let mut mut_assign = Vec::with_capacity(mut_fields.len());
            for (f, mut_field_t) in mut_fields.iter().enumerate() {
                let id = table
                    .fields
                    .get_by_right(&FieldIndex {
                        group: i,
                        mutable: true,
                        place: f,
                    })
                    .unwrap();
                field_params.push(quote! {#id: #mut_field_t});
                mut_assign.push(id);
            }
            group_assigns.push(quote! {
                self.groups.#i.put_new(((#(#imm_assign),*), (#(#mut_assign),*) ))
            });
        }

        let index_t = table.index.idx_t();

        OpRes {
            prelude: quote! { /* Insert Prelude */},
            method: quote! {
                fn #name(&mut self, #(#field_params),*) -> Result<#index_t::ExternalIndex, #index_t::NewIndexError> {
                    let idx = self.index.put_new()?;
                    #tks_before
                    #(#group_assigns;)*
                    #tks_after
                    Ok(idx)
                },
            },
        }
    }
}

pub struct Update {
    fields: Vec<FieldIndex>,
}

impl OpGen for Update {
    fn generate(
        &self,
        table: &Table,
        name: &OpName,
        tks_before: impl ToTokens,
        tks_after: impl ToTokens,
    ) -> OpRes {
        let index_t = table.index.idx_t();
        OpRes {
            prelude: quote! { ~TODO~ },
            method: quote! {
                    fn #name(&mut self, idx: #index_t::ExternalIndex, /* fields here */) -> Result<(), #index_t::NewIndexError> {
                        #tks_before
                        // update fields
                        #tks_after
                        Ok(())
                },
            },
        }
    }
}

pub struct Get {
    fields: Vec<FieldIndex>,
    name: FieldId,
}

impl OpGen for Get {
    fn generate(
        &self,
        table: &Table,
        name: &OpName,
        tks_before: impl ToTokens,
        tks_after: impl ToTokens,
    ) -> OpRes {
        let index_t = table.index.idx_t();
        OpRes {
            prelude: quote! { ~TODO~ },
            method: quote! {
                    fn #name(&self, idx: #index_t::ExternalIndex, /* fields here */) -> Result<(), #index_t::NewIndexError> {
                        #tks_before
                        // update fields
                        #tks_after
                        Ok(())
                },
            },
        }
    }
}

pub struct Brw {
    fields: Vec<FieldIndex>,
    name: FieldId,
}

impl OpGen for Brw {
    fn generate(
        &self,
        table: &Table,
        name: &OpName,
        tks_before: impl ToTokens,
        tks_after: impl ToTokens,
    ) -> OpRes {
        let index_t = table.index.idx_t();
        OpRes {
            prelude: quote! { ~TODO~ },
            method: quote! {
                    fn #name(&self, idx: #index_t::ExternalIndex, /* fields here */) -> Result<(), #index_t::NewIndexError> {
                        #tks_before
                        // update fields
                        #tks_after
                        Ok(())
                },
            },
        }
    }
}

pub struct BrwMut {
    fields: Vec<FieldIndex>,
    name: FieldId,
}

impl OpGen for BrwMut {
    fn generate(
        &self,
        table: &Table,
        name: &OpName,
        tks_before: impl ToTokens,
        tks_after: impl ToTokens,
    ) -> OpRes {
        let index_t = table.index.idx_t();
        OpRes {
            prelude: quote! { ~TODO~ },
            method: quote! {
                    fn #name(&mut self, idx: #index_t::ExternalIndex, /* fields here */) -> Result<(), #index_t::NewIndexError> {
                        #tks_before
                        // update fields
                        #tks_after
                        Ok(())
                },
            },
        }
    }
}

pub struct Count;

impl OpGen for Count {
    fn generate(
        &self,
        table: &Table,
        name: &OpName,
        tks_before: impl ToTokens,
        tks_after: impl ToTokens,
    ) -> OpRes {
        OpRes {
            prelude: quote! { ~TODO~ },
            method: quote! {
                fn #name(&self) -> usize {
                    #tks_before
                    let c = self.index.count();
                    #tks_after
                    c
                }
            },
        }
    }
}
