use std::collections::{HashMap, HashSet};
use quote_debug::Tokens;
use quote::quote;
use syn::{Ident, Item, ItemMod, ItemStruct};

use crate::{namer::Namer, ops::OperationKind, table::{PushVec, Table}};

pub struct Additionals {
    transactions: bool,
    debug: bool,
    unique: HashSet<Ident>,
}

//
//
/*

for each op make a module

pub mod insert_op {
    pub struct Arg {}
    pub struct ArgGet {}
    pub enum Error {}
}

impl ...  {

}


*/

/// Generate the struct 'Additonals' 
fn generate_additionals(
    table: &Table,
    namer: &Namer,
) -> Tokens<ItemMod> {
    
}