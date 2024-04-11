use proc_macro2::{Ident, Punct, Spacing, Span, TokenStream};
use quote::quote;

use super::super::Parseable;

#[derive(PartialEq, Eq, Debug)]
pub enum RecursiveIdent {
    Next {
        id: String,
        recur: Box<RecursiveIdent>,
    },
    Final,
}

impl Parseable for RecursiveIdent {
    type Param = usize;
    fn generate_case(param: Self::Param) -> Self {
        let mut case = RecursiveIdent::Final;
        for i in 0..param {
            case = RecursiveIdent::Next {
                id: format!("id{}", i),
                recur: Box::new(case),
            };
        }
        case
    }

    fn generate_tokens(&self) -> TokenStream {
        match self {
            RecursiveIdent::Next { id, recur } => {
                let id = Ident::new(id, Span::call_site());
                let recur = recur.generate_tokens();
                quote! { #id { #recur } }
            }
            RecursiveIdent::Final => {
                let p = Punct::new('!', Spacing::Alone);
                quote! { #p }
            }
        }
    }
}