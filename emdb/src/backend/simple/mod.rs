//! A simple reference backend, with no optimisations
//! - Uses generational arenas to store items
//! - Does not make use of any optimisations
//! - is ACID
//!
//!
//! ```text
//!
//! fn query_this(&mut self, args) {
//!     let precommit_log = empty;
//!
//!     fn try_query(self, precommit_log) -> Result<Ok, Err> {
//!         let mut op1 = scan(table)
//!         let mut op2 = filter(&mut op1, predicate)
//!         let mut op3 = map(&mut op2, blagh)
//!         
//!         let tab = multiply(&mut op3).run()?
//!         
//!         let access = scan(&mut tab)
//!         let mut op4 = filter(&mut access)
//!         let ret = map(&mut op4, lambda).run()?
//!         
//!         let mut op5 = update(&mut tab, lambda).run()?
//!         
//!         return ret
//!     }
//!
//!      // if error, undo commit log
//!      // if okay, return value
//! }
//! ```

use crate::{backend::Backend, plan::repr::LogicalPlan};
use proc_macro2::TokenStream;
use quote::quote;
pub(crate) struct Simple;

impl Backend for Simple {
    fn generate_code(plan: &LogicalPlan) -> TokenStream {
        quote! {}
    }
}
