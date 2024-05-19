use proc_macro2::Span;
use quote_debug::Tokens;
use syn::{Ident, Path};
use quote::quote;

pub struct CodeNamer;

impl CodeNamer {
    pub fn column_types_mod(&self) -> Ident {
        Ident::new("column_types", Span::call_site())
    }
    
    pub fn column_types_primary_mod(&self) -> Ident {
        Ident::new("primary", Span::call_site())
    }

    pub fn column_types_assoc_mod(&self, assoc_ind: usize) -> Ident {
        Ident::new(&format!("assoc_{assoc_ind}"), Span::call_site())
    }

    pub fn column_types_imm_struct(&self) -> Ident {
        Ident::new("Imm", Span::call_site())
    }

    pub fn column_types_mut_struct(&self) -> Ident {
        Ident::new("Mut", Span::call_site())
    }

    pub fn column_types_imm_unpacked_struct(&self) -> Ident {
        Ident::new("ImmUnpack", Span::call_site())
    }  

    pub fn column_types_imm_unpacked_fn(&self) -> Ident {
        Ident::new("imm_unpack", Span::call_site())
    }

    pub fn pulpit_path(&self) -> Tokens<Path> {
        quote!{pulpit}.into()
    } 

    pub fn column_holder(&self) -> Ident {
        Ident::new("ColumnHolder", Span::call_site())
    }

    pub fn window_holder(&self) -> Ident {
        Ident::new("WindowHolder", Span::call_site())
    }

    pub fn borrow_struct(&self) -> Ident {
        Ident::new("BorrowAll", Span::call_site())
    }

    pub fn borrow_trait(&self) -> Ident {
        Ident::new("BorrowAll", Span::call_site())
    }
}