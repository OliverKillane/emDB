use proc_macro2::TokenStream;

use crate::plan::repr::LogicalPlan;

mod volcano;
pub(crate) use volcano::Volcano;

pub(crate) trait Backend<'a> {
    fn generate_code(plan: LogicalPlan<'a>) -> TokenStream;
}
