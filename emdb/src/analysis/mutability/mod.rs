//! # Mutability Analysis
//! Determining how tables are updated in order to alter the access.

use crate::plan;
use std::collections::HashMap;

#[derive(Default)]
struct ColMut {
    read: bool,
    write: bool,
}

#[derive(Default)]
struct TableMut<'a> {
    insert: bool,
    delete: bool,
    reference: bool,
    per_col: HashMap<&'a plan::RecordField, ColMut>,
}

type TableAssocKey = usize;

struct Mutability<'a> {
    tables: HashMap<TableAssocKey, TableMut<'a>>,
}

impl<'a> Mutability<'a> {
    fn from_plan<'b>(lp: &'b plan::Plan) -> Mutability<'b> {
        let mut tables: HashMap<usize, TableMut<'b>> = lp
            .tables
            .iter()
            .map(|(key, table)| {
                (
                    key.to_idx(),
                    TableMut {
                        insert: false,
                        delete: false,
                        reference: false,
                        per_col: table
                            .columns
                            .keys()
                            .map(|id| {
                                (
                                    id,
                                    ColMut {
                                        read: false,
                                        write: false,
                                    },
                                )
                            })
                            .collect(),
                    },
                )
            })
            .collect();

        for (_, op) in lp.operators.iter() {
            match op {
                plan::Operator::Insert(plan::Insert { table, .. }) => {
                    tables.get_mut(&table.to_idx()).unwrap().insert = true;
                }
                plan::Operator::Delete(plan::Delete { table, .. }) => {
                    tables.get_mut(&table.to_idx()).unwrap().delete = true;
                }
                plan::Operator::UniqueRef(plan::UniqueRef { table, .. })
                | plan::Operator::ScanRefs(plan::ScanRefs { table, .. }) => {
                    tables.get_mut(&table.to_idx()).unwrap().reference = true;
                }
                plan::Operator::Update(plan::Update { table, mapping, .. }) => {
                    let tablemut = tables.get_mut(&table.to_idx()).unwrap();
                    for col in mapping.keys() {
                        tablemut.per_col.get_mut(col).unwrap().write = true;
                    }
                }
                plan::Operator::DeRef(plan::DeRef { table, .. }) => {
                    let tablemut = tables.get_mut(&table.to_idx()).unwrap();
                    for (_, m) in tablemut.per_col.iter_mut() {
                        m.read = true;
                    }
                }
                _ => (),
            }
        }

        Mutability { tables }
    }
}
