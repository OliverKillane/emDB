//! # Statistics
//! Types and struct generation for the statistics parameters for minister-based
//! operators.

use quote::quote;
use quote_debug::Tokens;
use syn::{ItemStruct, Type};
use super::{namer::SerializedNamer, operators::OperatorImpl};

#[derive(Copy, Clone)]
pub enum StatKind {
    Map,
    MapSeq,
    MapSingle,
    Filter,
    All,
    Is,
    Count,
    Fold,
    Combine,
    Sort,
    Take,
    GroupBy,
    CrossJoin,
    EquiJoin,
    PredJoin,
    Union,
    Fork,
    ForkSingle,

    // TODO: We intend to have some form of splitting for streams at some point, 
    //       hence its inclusion in minister, and the stats tag for it.
    #[allow(dead_code)]
    Split,
}

impl StatKind {
    fn datatype(&self) -> Tokens<Type> {
        match self {
            StatKind::Map => quote!(MapStats),
            StatKind::MapSeq => quote!(MapSeqStats),
            StatKind::MapSingle => quote!(MapSingleStats),
            StatKind::Filter => quote!(FilterStats),
            StatKind::All => quote!(AllStats),
            StatKind::Is => quote!(IsStats),
            StatKind::Count => quote!(CountStats),
            StatKind::Fold => quote!(FoldStats),
            StatKind::Combine => quote!(CombineStats),
            StatKind::Sort => quote!(SortStats),
            StatKind::Take => quote!(TakeStats),
            StatKind::GroupBy => quote!(GroupByStats),
            StatKind::CrossJoin => quote!(CrossJoinStats),
            StatKind::EquiJoin => quote!(EquiJoinStats),
            StatKind::PredJoin => quote!(PredJoinStats),
            StatKind::Union => quote!(UnionStats),
            StatKind::Fork => quote!(ForkStats),
            StatKind::ForkSingle => quote!(ForkSingleStats),
            StatKind::Split => quote!(SplitStats),
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
