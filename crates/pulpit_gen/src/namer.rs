use quote_debug::Tokens;
use quote::quote;
use syn::{ExprBlock, ExprClosure, Ident, ItemMod, Lifetime, Type};

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