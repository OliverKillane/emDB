use std::collections::HashMap;

use crate::{
    access::{AccessGen, AccessKind, FieldState},
    column::{AssocInd, ColumnGenerate, ColumnsConfig},
    ops::{OpGen, OperationKind},
};
use bimap::BiHashMap;
use proc_macro2::TokenStream;
use quote::quote;
use quote_debug::Tokens;
use syn::{ExprBlock, Ident, ItemMod, Lifetime, Type};

pub struct Table {
    pub columns: ColumnsConfig,
    pub access: Vec<AccessKind>,
    pub transactions: bool,
    pub operations: HashMap<Ident, OperationKind>,
}

pub struct Namer {
    pub mod_name: Ident,
}
impl Namer {
    pub fn mod_name(&self) -> &Ident {
        &self.mod_name
    }
    pub fn access_field(&self) -> Ident {
        Ident::new("access_fields", proc_macro2::Span::call_site())
    }
    pub fn col_field(&self, id: usize) -> Ident {
        Ident::new(&format!("col_{}", id), proc_macro2::Span::call_site())
    }
    pub fn window_lifetime(&self) -> Tokens<Lifetime> {
        quote!('imm).into()
    }
    pub fn primary_column(&self) -> Ident {
        Ident::new("primary_column", proc_macro2::Span::call_site())
    }

    pub fn associated_column_tuple(&self) -> Ident {
        Ident::new("assoc_columns", proc_macro2::Span::call_site())
    }
}

pub struct Hook {
    before: Vec<Tokens<ExprBlock>>,
    after: Vec<Tokens<ExprBlock>>,
}

pub struct HookStore {
    #[allow(clippy::type_complexity)]
    hooks: HashMap<Ident, Hook>,
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

impl Default for HookStore {
    fn default() -> Self {
        Self::new()
    }
}

impl HookStore {
    pub fn new() -> Self {
        Self {
            hooks: HashMap::new(),
        }
    }
    pub fn push_hook(
        &mut self,
        operation: &Ident,
        before: Tokens<ExprBlock>,
        after: Tokens<ExprBlock>,
    ) {
        if self.hooks.contains_key(operation) {
            let prev_hooks = self.hooks.get_mut(operation).unwrap();
            prev_hooks.before.push(before);
            prev_hooks.after.push(after);
        } else {
            self.hooks.insert(
                operation.clone(),
                Hook {
                    before: vec![before],
                    after: vec![after],
                },
            );
        }
    }
    pub fn get_hooks(&self, operation: &Ident) -> Option<&Hook> {
        self.hooks.get(operation)
    }
}

impl Table {
    pub fn generate(&self, namer: &Namer) -> Tokens<ItemMod> {
        let mut hooks = HookStore::new();
        let mut prelude = PushVec::new();
        let mut methods = PushVec::new();

        let (access_fields_types, access_fields_init): (Vec<_>, Vec<_>) = self
            .access
            .iter()
            .enumerate()
            .map(|(access_id, access)| {
                access.gen(
                    access_id,
                    namer,
                    self,
                    &mut hooks,
                    &mut prelude,
                    &mut methods,
                )
            })
            .map(|FieldState { datatype, init }| (datatype, init))
            .unzip();
        let access_fields_name = namer.access_field();

        let primary_type = self
            .columns
            .primary_col
            .col
            .generate(&self.columns.primary_col.fields, &mut prelude);
        let primary_name = namer.primary_column();

        let (assoc_fields_types, assoc_field_numbers): (Vec<_>, Vec<_>) = self
            .columns
            .assoc_columns
            .iter()
            .enumerate()
            .map(|(ind, assoc_col)| (assoc_col.col.generate(&assoc_col.fields, &mut prelude), ind))
            .unzip();
        let assoc_cols_name = namer.associated_column_tuple();

        for (id, op) in &self.operations {
            let (before, after) = if let Some(Hook { before, after }) = hooks.get_hooks(id) {
                (before, after)
            } else {
                (&Vec::new(), &Vec::new())
            };
            methods.push(op.generate(id, self, namer, &mut prelude, before, after));
        }

        let mod_name = namer.mod_name();
        let prelude = prelude.open();
        let methods = methods.open();
        let window_lifetime = namer.window_lifetime();

        quote! {
            mod #mod_name {
                #(#prelude)*

                struct Table {
                    #access_fields_name : (#(#access_fields_types),*),
                    #primary_name : #primary_type,
                    #assoc_cols_name : (#(#assoc_fields_types),*),
                }

                struct Window<#window_lifetime> {
                    #access_fields_name : &#window_lifetime mut (#(#access_fields_types),*),
                    #primary_name : <#primary_type as Column>::WindowKind<#window_lifetime>,
                    #assoc_cols_name : (#(<#assoc_fields_types as Column>::WindowKind<#window_lifetime> ),*),
                }

                impl Table {
                    fn new() -> Self {
                        Self {
                            #access_fields_name : (#(#access_fields_init),*),
                            #primary_name: <#primary_type as Column>::new(0),
                            #assoc_cols_name : (#(<#assoc_fields_types as Column>::new(0)),*),
                        }
                    }
                    fn window(&mut self) -> Window<'_> {
                        let Self {
                            #access_fields_name,
                            #primary_name,
                            #assoc_cols_name,
                        } = self;
                        Window {
                            #access_fields_name: #access_fields_name,
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
