use std::collections::HashMap;

use crate::{
    access::{AccessGen, AccessKind, FieldState},
    column::{ColumnGen, Columns, Group},
    index::{IndexGen, IndexKind},
    ops::{OpGen, OperationKind},
};
use bimap::BiHashMap;
use proc_macro2::TokenStream;
use quote::quote;
use quote_debug::Tokens;
use syn::{ExprBlock, Ident, ItemMod, Type};

pub struct Table {
    pub namer: Namer,
    pub columns: Columns,
    pub index: IndexKind,
    pub access: Vec<AccessKind>,
    pub operations: HashMap<Ident, OperationKind>,
}

pub struct Namer {
    mod_name: Ident,
}
impl Namer {
    pub fn mod_name(&self) -> &Ident {
        &self.mod_name
    }
    pub fn access_field(&self, id: usize) -> Ident {
        Ident::new(
            &format!("access_field_{}", id),
            proc_macro2::Span::call_site(),
        )
    }
    pub fn col_field(&self, id: usize) -> Ident {
        Ident::new(&format!("col_{}", id), proc_macro2::Span::call_site())
    }
    pub fn window_lifetime(&self) -> Ident {
        Ident::new("'imm", proc_macro2::Span::call_site())
    }
    pub fn index_field(&self) -> Ident {
        Ident::new("index", proc_macro2::Span::call_site())
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
    fn generate(&self) -> Tokens<ItemMod> {
        let mut hooks = HookStore::new();
        let mut prelude = PushVec::new();
        let mut methods = PushVec::new();

        let (fields_def, fields_init): (Vec<_>, Vec<_>) = self
            .access
            .iter()
            .enumerate()
            .map(|(access_id, access)| {
                let access_ident = self.namer.access_field(access_id);
                (
                    access.gen(&access_ident, self, &mut hooks, &mut prelude, &mut methods),
                    access_ident,
                )
            })
            .chain(self.columns.groups().map(
                |(
                    col_id,
                    Group {
                        col,
                        mut_fields,
                        imm_fields,
                    },
                )| {
                    let col_ident = self.namer.col_field(col_id);
                    (col.gen_state(&col_ident, mut_fields, imm_fields), col_ident)
                },
            ))
            .map(|(FieldState { datatype, init }, access_ident)| {
                (
                    quote!(#access_ident : #datatype ),
                    quote!(#access_ident : #init),
                )
            })
            .unzip();

        for (id, op) in &self.operations {
            let (before, after) = if let Some(Hook { before, after }) = hooks.get_hooks(id) {
                (before, after)
            } else {
                (&Vec::new(), &Vec::new())
            };
            methods.push(op.generate(id, self, &mut prelude, before, after));
        }

        let mod_name = self.namer.mod_name();
        let prelude = prelude.open();
        let methods = methods.open();
        let window_lifetime = self.namer.window_lifetime();
        let FieldState { datatype, init } = self.index.gen_state();
        let index_id = self.namer.index_field();

        quote! {
            mod #mod_name {
                #(#prelude)*

                struct Table {
                    #(#fields_def),*
                    #index_id : #datatype
                }

                impl Table {
                    fn new() -> Self {
                        Self {
                            #(#fields_init),*
                            #index_id : #init
                        }
                    }
                    fn window(&mut self) -> Window<'_> {
                        Window {
                            table: self
                        }
                    }
                }

                struct Window<#window_lifetime> {
                    table: &#window_lifetime Table
                }
                impl <#window_lifetime> Window<#window_lifetime> {
                    #(#methods)*
                }
            }
        }
        .into()
    }
}
