use crate::plan::repr::LogicalPlan;

use super::Backend;
use proc_macro2::TokenStream;
use quote::quote;

pub(crate) struct GraphViz;

impl Backend for GraphViz {
    fn generate_code(plan: &LogicalPlan) -> TokenStream {
        quote! { pub fn say_cool(x: bool) -> i32 { 3} }
    }
}
