//! # Statistics
//! Types and struct generation for the statistics parameters for minister-based
//! operators.

use quote::quote;
use quote_debug::Tokens;
use syn::{ItemStruct, Type};
use super::{namer::SerializedNamer, operators::OperatorImpl};

#[derive(Copy, Clone)]
pub enum StatKind {
    MapStats,
    MapSeqStats,
    MapSingleStats,
    FilterStats,
    AllStats,
    IsStats,
    CountStats,
    FoldStats,
    CombineStats,
    SortStats,
    TakeStats,
    GroupByStats,
    CrossJoinStats,
    EquiJoinStats,
    PredJoinStats,
    UnionStats,
    ForkStats,
    ForkSingleStats,
    SplitStats,
}

impl StatKind {
    fn datatype(&self) -> Tokens<Type> {
        match self {
            StatKind::MapStats => quote!(MapStats),
            StatKind::MapSeqStats => quote!(MapSeqStats),
            StatKind::MapSingleStats => quote!(MapSingleStats),
            StatKind::FilterStats => quote!(FilterStats),
            StatKind::AllStats => quote!(AllStats),
            StatKind::IsStats => quote!(IsStats),
            StatKind::CountStats => quote!(CountStats),
            StatKind::FoldStats => quote!(FoldStats),
            StatKind::CombineStats => quote!(CombineStats),
            StatKind::SortStats => quote!(SortStats),
            StatKind::TakeStats => quote!(TakeStats),
            StatKind::GroupByStats => quote!(GroupByStats),
            StatKind::CrossJoinStats => quote!(CrossJoinStats),
            StatKind::EquiJoinStats => quote!(EquiJoinStats),
            StatKind::PredJoinStats => quote!(PredJoinStats),
            StatKind::UnionStats => quote!(UnionStats),
            StatKind::ForkStats => quote!(ForkStats),
            StatKind::ForkSingleStats => quote!(ForkSingleStats),
            StatKind::SplitStats => quote!(SplitStats),
        }
        .into()
    }
}

pub struct RequiredStats {
    all: Vec<StatKind>,
}

impl RequiredStats {
    pub fn new() -> Self {
        Self { all: Vec::new() }
    }
 
    pub fn add_stat(&mut self, kind: StatKind) -> usize {
        let id = self.all.len();
        self.all.push(kind);
        id
    }

    pub fn generate_stats_struct(
        &self,
        namer @ SerializedNamer { struct_stats, .. }: &SerializedNamer,
        OperatorImpl { impl_alias, trait_path }: &OperatorImpl,
    ) -> Tokens<ItemStruct> {
        let members = self.all.iter().enumerate().map(|(index, kind)| {
            let name = namer.name_stat_member(index);
            let kind = kind.datatype();
            quote! { #name: <#impl_alias as #trait_path>::#kind }
        });
        quote! {
            #[derive(Default)]
            struct #struct_stats {
                #(#members),*
            }
        }
        .into()
    }
}
