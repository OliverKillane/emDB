
use std::{collections::HashMap, fs::File, io::Write, path::Path};
use prettyplease::unparse;
use proc_macro2::Span;
use pulpit_gen::{access::DebugAccess, column::{ColFields, Column, ColumnsConfig, PrimaryColumn, PrimaryRetain}, ops::Insert, table::{Namer, Table}};
use quote::ToTokens;
use syn::{parse2, Ident};

#[test]
fn main() {
    let table = Table{ 
        columns: ColumnsConfig { primary_col: Column{ col: PrimaryRetain{block_size: 10}.into(), fields:  ColFields { imm_data: vec![], mut_data: vec![] }}, assoc_columns: vec![] }, 
        access: vec![DebugAccess.into()], 
        transactions: false, 
        user_ops: HashMap::from([(Ident::new("cool_insert", Span::call_site()), Insert.into())]) 
    };

    let tks = table.generate(&Namer{mod_name: Ident::new("cool_mod", Span::call_site())});
    

    println!("{}", unparse(&parse2(tks.into_token_stream()).unwrap()))
}

