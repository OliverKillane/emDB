//! A simple reference backend using basic volcano operators

use super::EMDBBackend;
mod ops;
pub struct Simple{}

impl EMDBBackend for Simple {
    const NAME: &'static str = "Simple";

    fn parse_options(
        backend_name: &syn::Ident,
        options: Option<proc_macro2::TokenStream>,
    ) -> Result<Self, std::collections::LinkedList<proc_macro_error::Diagnostic>> {
        todo!()
    }

    fn generate_code(
        self,
        impl_name: syn::Ident,
        plan: &crate::plan::Plan,
    ) -> Result<proc_macro2::TokenStream, std::collections::LinkedList<proc_macro_error::Diagnostic>> {
        todo!()
    }
}