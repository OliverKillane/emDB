use crate::{backend::Backend, plan::repr::LogicalPlan};
use proc_macro2::TokenStream;
use quote::quote;
pub(crate) struct Simple;

impl Backend for Simple {
    fn generate_code(plan: &LogicalPlan) -> TokenStream {
        quote! {}
    }
}
