use prettyplease::unparse;
use proc_macro2::Span;
use pulpit_gen::{
    columns::{
        Append, AppendTrans, AssocBlocks, AssocVec, PrimaryNoPull, PrimaryRetain,
        PrimaryThunderdome, Pull, PullTrans,
    },
    groups::{Field, Group, GroupConfig, MutImmut},
    namer::CodeNamer,
    operations::update::Update,
    predicates::Predicate,
    table::Table,
    uniques::Unique,
};
use quote::quote;
use quote::ToTokens;
use std::{collections::HashSet, fs};
use syn::{parse2, Ident};

#[test]
fn main() {
    let a = Ident::new("a", Span::call_site());
    let b = Ident::new("b", Span::call_site());
    let c = Ident::new("c", Span::call_site());
    let d = Ident::new("d", Span::call_site());
    let e = Ident::new("e", Span::call_site());

    let table = Table::<Append> {
        groups: GroupConfig {
            primary: Group {
                col: PrimaryNoPull(AssocBlocks { block_size: 1024 }.into()).into(),
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
                col: AssocVec.into(),
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
        uniques: vec![
            Unique {
                alias: Ident::new("e_unique", Span::call_site()),
                field: e.clone(),
            },
            Unique {
                alias: Ident::new("a_unique", Span::call_site()),
                field: a.clone(),
            },
        ],
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
                fields: vec![a.clone(), c.clone(), e.clone()],
                alias: Ident::new("update_ace", Span::call_site()),
            },
            Update {
                fields: vec![a.clone()],
                alias: Ident::new("update_a", Span::call_site()),
            },
        ],
        name: Ident::new("my_table", Span::call_site()),
    };

    let tks = table.generate(&CodeNamer::new());

    fs::write(
        "../pulpit/tests/output.rs",
        unparse(&parse2(tks.into_token_stream()).unwrap()),
    )
    .unwrap();
}
