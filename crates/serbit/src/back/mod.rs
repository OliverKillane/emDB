use quote_debug::Tokens;
use syn::ItemMod;

use crate::plan::ir;
pub mod chew;

pub trait Backend<'consts, 'datas, 'stages, N: ir::namer::Namer> {
    type Error;

    fn generate(plan: &ir::Plan<'consts, 'datas, 'stages, N>) -> Result<Tokens<ItemMod>, Self::Error>;
}

