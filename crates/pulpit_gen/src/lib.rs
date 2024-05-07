#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
use bimap::BiHashMap;
use proc_macro2::{Ident, TokenStream};
use quote::quote;

use crate::{
    access::AccessGen,
    column::Gen,
    index::IndexGen,
    ops::{OpGen, OpRes},
};

type FieldType = TokenStream;
type FieldId = Ident;
type OpName = Ident;

pub mod access;
pub mod column;
pub mod index;
pub mod ops;

pub struct Group {
    pub col: column::Kind,
    pub mut_fields: Vec<FieldType>,
    pub imm_fields: Vec<FieldType>,
}

#[derive(Hash, Eq, PartialEq)]
pub struct FieldIndex {
    group: usize,
    mutable: bool,
    place: usize,
}

pub struct Table {
    name: Ident,
    groups: Vec<Group>,
    fields: BiHashMap<Ident, FieldIndex>,
    index: index::Kind,
    access: Vec<access::Kind>,
    ops: Vec<(ops::Kind, Ident)>,
}

impl Table {
    fn generate_code(&self) -> TokenStream {
        let groups: Vec<_> = self
            .groups
            .iter()
            .map(|grp| grp.col.generate(&grp.mut_fields, &grp.imm_fields))
            .collect();
        let index = self.index.generate();
        let access_states = self.access.iter().map(AccessGen::state);

        let (prelude, methods): (Vec<_>, Vec<_>) = self
            .ops
            .iter()
            .map(|(op, id)| op.generate(self, id, quote! {}, quote! {}))
            .map(|OpRes { prelude, method }| (prelude, method))
            .unzip();

        let mod_name = &self.name;
        quote! {
            mod #mod_name {
                struct Table {
                    groups: (#(#groups),*),
                    index: #index,
                    access: (#(#access_states),*),
                }

                struct Window<'imm> {
                    table: &'imm Table
                }

                #(#prelude)*

                impl <'imm> Window<'imm> {
                    #(#methods)*
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_output() {
        // Table {
        //     name: Ident::new("my_table", Span::call_site()),
        //     groups: vec![Group { col: column::VecCol.into(), mut_fields: vec![quote!{String}], imm_fields: vec![quote!{Coolio<i32>}] }, Group {col: column::VecCol}],
        //     fields: todo!(),
        //     index: todo!(),
        //     access: todo!(),
        //     ops: todo!(),
        // };
    }
}
