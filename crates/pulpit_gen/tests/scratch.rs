use prettyplease::unparse;
use proc_macro2::Span;
use pulpit_gen::{
    columns::{AssocVec, PrimaryRetain},
    groups::{Field, Group, GroupConfig, Groups, MutImmut},
    namer::CodeNamer,
    operations::update::Update,
    predicates::Predicate,
    table::Table,
    uniques::Unique,
};
use quote::quote;
use quote::ToTokens;
use std::{
    collections::{HashMap, HashSet},
    fs,
};
use syn::{parse2, Ident};

#[test]
fn main() {
    let a = Ident::new("a", Span::call_site());
    let b = Ident::new("b", Span::call_site());
    let c = Ident::new("c", Span::call_site());
    let d = Ident::new("d", Span::call_site());
    let e = Ident::new("e", Span::call_site());

    let table = Table {
        groups: GroupConfig {
            primary: Group {
                col: PrimaryRetain { block_size: 1024 },
                fields: MutImmut {
                    imm_fields: vec![Field {
                        name: b.clone(),
                        ty: quote! {usize}.into(),
                    }],
                    mut_fields: vec![
                        Field {
                            name: a.clone(),
                            ty: quote! {i32}.into(),
                        },
                        Field {
                            name: c.clone(),
                            ty: quote! {Option<String>}.into(),
                        },
                    ],
                },
            },
            assoc: vec![Group {
                col: AssocVec,
                fields: MutImmut {
                    imm_fields: vec![Field {
                        name: d.clone(),
                        ty: quote! {char}.into(),
                    }],
                    mut_fields: vec![Field {
                        name: e.clone(),
                        ty: quote! {String}.into(),
                    }],
                },
            }],
        }
        .into(),
        uniques: HashMap::from([
            (
                e.clone(),
                Unique {
                    alias: Ident::new("e_unique", Span::call_site()),
                },
            ),
            (
                a.clone(),
                Unique {
                    alias: Ident::new("a_unique", Span::call_site()),
                },
            ),
        ]),
        predicates: vec![
            Predicate {
                alias: Ident::new("check_b", Span::call_site()),
                tokens: quote! {*b < 1045}.into(),
            },
            Predicate {
                alias: Ident::new("check_e_len", Span::call_site()),
                tokens: quote! {e.len() > *b}.into(),
            },
        ],
        updates: vec![
            Update {
                fields: HashSet::from([a.clone(), c.clone(), e.clone()]),
                alias: Ident::new("update_ace", Span::call_site()),
            },
            Update {
                fields: HashSet::from([a.clone()]),
                alias: Ident::new("update_a", Span::call_site()),
            },
        ],
        name: Ident::new("my_table", Span::call_site()),
    };

    let tks = table.generate(&CodeNamer);

    fs::write(
        "../pulpit/tests/output.rs",
        unparse(&parse2(tks.into_token_stream()).unwrap()),
    )
    .unwrap();
}

// fn update_ace (& mut self , update : Update , key : Key) -> Result < () , UpdateError > { let Entry { index , data : primary } = match self . columns . primary . brw_mut (key) { Ok (entry) => entry , Err (e) => return Err (UpdateError :: KeyError) , } let assoc_0 = self . columns . assoc_0 . brw_mut (index) ; if ! predicates :: check_b (& primary . imm_data . b , update . a , update . c , update . e , & assoc_0 . mut_data . d) { return Err (UpdateError :: check_b) ; } if ! predicates :: check_e_len (& primary . imm_data . b , update . a , update . c , update . e , & assoc_0 . mut_data . d) { return Err (UpdateError :: check_e_len) ; } let e_unique = match self . uniques . e_unique . replace (& update . e , & assoc_0 . imm_data . e , key) { Ok (old_val) => old_val , Err (_) => { return Err (UpdateError :: e_unique) } , } ; let mut update = update ; std :: mem :: swap (& mut primary . mut_data . c , & mut update . c) ; ; std :: mem :: swap (& mut assoc_0 . mut_data . e , & mut update . e) ; ; std :: mem :: swap (& mut primary . mut_data . a , & mut update . a) ; ; if self . transactions . append { self . transactions . log . push (transactions :: LogItem :: Update (transactions :: Updates :: update_ace (update))) ; } Ok (()) }
