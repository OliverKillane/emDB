//! For converting a selection of operations into a concrete one for code generation.

// choice of deletion, transaction, updates and mutability.

pub struct SelectOperations {
    pub name: Ident,
    pub transactions: bool,
    pub deletions: bool,
    pub fields: HashMap<Ident, Tokens<Type>>,
    pub uniques: Vec<Unique>,
    pub predicates: Vec<Predicate>,
    pub updates: Vec<Update>,
    pub public: bool,
}

use std::collections::HashMap;

use quote_debug::Tokens;
use syn::{Ident, ItemMod, Type};

use crate::{
    columns::{Append, AppendTrans, AssocBlocks, PrimaryNoPull, PrimaryRetain, Pull, PullTrans},
    groups::{Field, Group, GroupConfig, MutImmut},
    namer::CodeNamer,
    operations::update::Update,
    predicates::Predicate,
    table::Table,
    uniques::Unique,
};

#[enumtrait::quick_from]
#[enumtrait::store(enum_backing_kind)]
pub enum BackingKind {
    Append(Table<Append>),
    AppendTrans(Table<AppendTrans>),
    Pull(Table<Pull>),
    PullTrans(Table<PullTrans>),
}

impl BackingKind {
    pub fn generate(self, namer: &CodeNamer) -> Tokens<ItemMod> {
        enumtrait::gen_match!(enum_backing_kind as self for b => b.generate(namer))
    }

    pub fn op_get_types(&self, namer: &CodeNamer) -> HashMap<Ident, Tokens<Type>> {
        enumtrait::gen_match!(enum_backing_kind as self for b => b.op_get_types(namer))
    }

    pub fn insert_can_error(&self) -> bool {
        enumtrait::gen_match!(enum_backing_kind as self for b => b.insert_can_error())
    }
}

pub fn select_basic(
    SelectOperations {
        name,
        transactions,
        deletions,
        mut fields,
        uniques,
        predicates,
        updates,
        public,
    }: SelectOperations,
) -> BackingKind {
    fn convert_fields(fields: HashMap<Ident, Tokens<Type>>) -> Vec<Field> {
        fields
            .into_iter()
            .map(|(name, ty)| Field { name, ty })
            .collect()
    }

    let mut mut_fields = HashMap::new();
    for Update {
        fields: update_fields,
        alias: _,
    } in &updates
    {
        for field in update_fields {
            if let Some(ty) = fields.remove(field) {
                mut_fields.insert(field.clone(), ty);
            }
        }
    }
    let primary_fields = MutImmut {
        imm_fields: convert_fields(fields),
        mut_fields: convert_fields(mut_fields),
    };

    if transactions {
        if deletions {
            Table::<PullTrans> {
                groups: GroupConfig {
                    primary: Group {
                        col: PrimaryRetain { block_size: 1024 }.into(),
                        fields: primary_fields,
                    },
                    assoc: vec![],
                }
                .into(),
                uniques,
                predicates,
                updates,
                name,
                public,
            }
            .into()
        } else {
            Table::<AppendTrans> {
                groups: GroupConfig {
                    primary: Group {
                        col: PrimaryNoPull(AssocBlocks { block_size: 1024 }.into()).into(),
                        fields: primary_fields,
                    },
                    assoc: vec![],
                }
                .into(),
                uniques,
                predicates,
                updates,
                name,
                public,
            }
            .into()
        }
    } else if deletions {
        Table::<Pull> {
            groups: GroupConfig {
                primary: Group {
                    col: PrimaryRetain { block_size: 1024 }.into(),
                    fields: primary_fields,
                },
                assoc: vec![],
            }
            .into(),
            uniques,
            predicates,
            updates,
            name,
            public,
        }
        .into()
    } else {
        Table::<Append> {
            groups: GroupConfig {
                primary: Group {
                    col: PrimaryNoPull(AssocBlocks { block_size: 1024 }.into()).into(),
                    fields: primary_fields,
                },
                assoc: vec![],
            }
            .into(),
            uniques,
            predicates,
            updates,
            name,
            public,
        }
        .into()
    }
}
