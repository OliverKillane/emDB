mod emql;
mod sql;

use crate::plan::repr::LogicalPlan;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::Diagnostic;
trait Frontend<'a> {
    fn from_tokens(input: TokenStream2) -> Result<LogicalPlan<'a>, Vec<Diagnostic>>;
}
