use crate::{backend::Backend, plan::repr::LogicalPlan};
use proc_macro2::TokenStream;
pub(crate) struct Volcano;

impl<'a> Backend<'a> for Volcano {
    fn generate_code(plan: LogicalPlan<'a>) -> TokenStream {
        todo!()
    }
}
