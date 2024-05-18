use std::collections::HashMap;

use crate::{
    additions::Additionals, column::{AssocInd, ColumnGenerate, ColumnTypes, ColumnsConfig, FieldState}, namer::Namer, ops::{OpGen, OperationKind}
};
use bimap::BiHashMap;
use proc_macro2::TokenStream;
use quote::quote;
use quote_debug::Tokens;
use syn::{ExprBlock, ExprClosure, Ident, ItemMod, Lifetime, Type};

pub struct Table {
    pub columns: ColumnsConfig,
    pub additions: Additionals,
    pub user_ops: HashMap<Ident, OperationKind>,
    pub access_ops: Vec<OperationKind>,
}

pub struct PushVec<T>(Vec<T>);
impl<T> Default for PushVec<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> PushVec<T> {
    pub fn new() -> Self {
        Self(Vec::new())
    }
    pub fn push(&mut self, data: T) {
        self.0.push(data)
    }
    pub fn open(self) -> Vec<T> {
        self.0
    }
}

impl Table {
    pub fn generate(&self, namer: &Namer) -> Tokens<ItemMod> {
        let mut prelude = PushVec::new();
        let mut methods = PushVec::new();

        let ColumnTypes{ concrete_type: primary_type, kind_trait:primarykind, access_trait:_ } = self
            .columns
            .primary_col
            .col
            .generate(namer, &self.columns.primary_col.fields, &mut prelude);
        let primary_name = namer.primary_column();

        let (assoc_fields_types, assoc_field_numbers): (Vec<_>, Vec<_>) = self
            .columns
            .assoc_columns
            .iter()
            .enumerate()
            .map(|(ind, assoc_col)| (assoc_col.col.generate(namer, &assoc_col.fields, &mut prelude).concrete_type, ind))
            .unzip();
        let assoc_cols_name = namer.associated_column_tuple();

        for (id, op) in &self.user_ops {
            methods.push(op.generate(id, self, namer, &mut prelude));
        }

        let mod_name = namer.mod_name();
        let prelude = prelude.open();
        let methods = methods.open();
        let window_lifetime = namer.window_lifetime();

        quote! {
            mod #mod_name {
                #(#prelude)*

                pub type Key<#window_lifetime> = <<#primary_type as Column>::WindowKind<#window_lifetime> as #primarykind>::Key;

                pub struct Table {
                    #primary_name : #primary_type,
                    #assoc_cols_name : (#(#assoc_fields_types),*),
                }

                pub struct Window<#window_lifetime> {
                    #primary_name : <#primary_type as Column>::WindowKind<#window_lifetime>,
                    #assoc_cols_name : (#(<#assoc_fields_types as Column>::WindowKind<#window_lifetime> ),*),
                }

                impl Table {
                    pub fn new() -> Self {
                        Self {
                            #primary_name: <#primary_type as Column>::new(0),
                            #assoc_cols_name : (#(<#assoc_fields_types as Column>::new(0)),*),
                        }
                    }
                    pub fn window(&mut self) -> Window<'_> {
                        let Self {
                            #primary_name,
                            #assoc_cols_name,
                        } = self;
                        Window {
                            #primary_name: #primary_name.window(),
                            #assoc_cols_name: (#(#assoc_cols_name.#assoc_field_numbers.window()),*),
                        }
                    }
                }
                impl <#window_lifetime> Window<#window_lifetime> {
                    #(#methods)*
                }
            }
        }
        .into()
    }
}
